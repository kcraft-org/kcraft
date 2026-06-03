use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PluginError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Failed to load plugin: {0}")]
    LoadFailed(String),
    #[error("Plugin version mismatch: {0}")]
    VersionMismatch(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, PluginError>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub api_version: String,
    pub entry_point: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PluginHandle {
    pub manifest: PluginManifest,
    pub enabled: bool,
    pub load_time: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginState {
    Loaded,
    Enabled,
    Disabled,
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PluginEvent {
    PreLaunch,
    PostLaunch,
    InstanceCreated,
    InstanceDeleted,
    ModInstalled,
    GameLaunching { instance_id: String },
}

type EventCallback = Box<dyn Fn(&PluginEvent) + Send + Sync>;

pub struct PluginRegistry {
    plugins_dir: PathBuf,
    handles: Vec<PluginHandle>,
    hooks: HashMap<PluginEvent, Vec<EventCallback>>,
}

impl PluginRegistry {
    pub fn new(plugins_dir: PathBuf) -> Self {
        Self {
            plugins_dir,
            handles: Vec::new(),
            hooks: HashMap::new(),
        }
    }

    pub fn discover_plugins(&mut self) -> Result<Vec<PluginHandle>> {
        let mut discovered = Vec::new();

        if !self.plugins_dir.exists() {
            std::fs::create_dir_all(&self.plugins_dir)?;
            return Ok(discovered);
        }

        for entry in std::fs::read_dir(&self.plugins_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().is_some_and(|ext| ext == "json") {
                let content = std::fs::read_to_string(&path)?;
                let manifest: PluginManifest = serde_json::from_str(&content)?;

                let handle = PluginHandle {
                    manifest,
                    enabled: false,
                    load_time: Duration::default(),
                };

                tracing::debug!(
                    "Discovered plugin: {} v{}",
                    handle.manifest.name,
                    handle.manifest.version
                );
                discovered.push(handle);
            }
        }

        self.handles = discovered.clone();
        Ok(discovered)
    }

    pub fn load_plugin(&mut self, manifest: &PluginManifest) -> Result<PluginHandle> {
        let handle = PluginHandle {
            manifest: manifest.clone(),
            enabled: false,
            load_time: Duration::default(),
        };

        let start = std::time::Instant::now();
        let load_time = start.elapsed();

        let handle = PluginHandle {
            load_time,
            ..handle
        };

        let existing = self
            .handles
            .iter()
            .position(|h| h.manifest.name == manifest.name);
        if let Some(idx) = existing {
            self.handles[idx] = handle.clone();
        } else {
            self.handles.push(handle.clone());
        }

        tracing::info!("Loaded plugin: {} v{}", manifest.name, manifest.version);
        Ok(handle)
    }

    pub fn enable_plugin(&mut self, name: &str) -> Result<()> {
        let handle = self
            .handles
            .iter_mut()
            .find(|h| h.manifest.name == name)
            .ok_or_else(|| PluginError::LoadFailed(format!("plugin '{name}' not found")))?;

        handle.enabled = true;
        tracing::info!("Enabled plugin: {name}");
        Ok(())
    }

    pub fn disable_plugin(&mut self, name: &str) -> Result<()> {
        let handle = self
            .handles
            .iter_mut()
            .find(|h| h.manifest.name == name)
            .ok_or_else(|| PluginError::LoadFailed(format!("plugin '{name}' not found")))?;

        handle.enabled = false;
        tracing::info!("Disabled plugin: {name}");
        Ok(())
    }

    pub fn uninstall_plugin(&mut self, name: &str) -> Result<()> {
        let idx = self
            .handles
            .iter()
            .position(|h| h.manifest.name == name)
            .ok_or_else(|| PluginError::LoadFailed(format!("plugin '{name}' not found")))?;

        let manifest_path = self.plugins_dir.join(format!("{name}.json"));
        if manifest_path.exists() {
            std::fs::remove_file(&manifest_path)?;
        }

        self.handles.remove(idx);
        tracing::info!("Uninstalled plugin: {name}");
        Ok(())
    }

    pub fn list_plugins(&self) -> Vec<PluginHandle> {
        self.handles.clone()
    }

    pub fn dispatch_event(&self, event: &PluginEvent) {
        if let Some(callbacks) = self.hooks.get(event) {
            for callback in callbacks {
                callback(event);
            }
        }
    }

    pub fn register_hook(&mut self, event: PluginEvent, callback: EventCallback) {
        self.hooks.entry(event).or_default().push(callback);
    }
}

pub struct PluginBuilder {
    manifest: PluginManifest,
}

impl PluginBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            manifest: PluginManifest {
                name: name.to_string(),
                version: "1.0.0".to_string(),
                author: String::new(),
                description: String::new(),
                api_version: "1.0".to_string(),
                entry_point: format!("plugins/{name}/main.js"),
                permissions: Vec::new(),
            },
        }
    }

    pub fn version(mut self, version: &str) -> Self {
        self.manifest.version = version.to_string();
        self
    }

    pub fn author(mut self, author: &str) -> Self {
        self.manifest.author = author.to_string();
        self
    }

    pub fn description(mut self, description: &str) -> Self {
        self.manifest.description = description.to_string();
        self
    }

    pub fn api_version(mut self, api_version: &str) -> Self {
        self.manifest.api_version = api_version.to_string();
        self
    }

    pub fn entry_point(mut self, entry_point: &str) -> Self {
        self.manifest.entry_point = entry_point.to_string();
        self
    }

    pub fn permission(mut self, permission: &str) -> Self {
        self.manifest.permissions.push(permission.to_string());
        self
    }

    pub fn build(self) -> PluginManifest {
        self.manifest
    }

    pub fn save(self, path: impl AsRef<Path>) -> Result<PluginManifest> {
        let manifest = self.build();
        let json = serde_json::to_string_pretty(&manifest)?;
        std::fs::write(path.as_ref(), json)?;
        Ok(manifest)
    }
}
