use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceType {
    Unknown,
    Folder,
    ZipFile,
    Litemod,
    SingleFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnableAction {
    Enable,
    Disable,
    Toggle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortType {
    Name,
    Date,
    Enabled,
}

#[derive(Debug, Clone)]
pub struct Resource {
    path: PathBuf,
    changed_date_time: i64,
    internal_id: String,
    name: String,
    resource_type: ResourceType,
    enabled: bool,
    is_resolving: bool,
    is_resolved: bool,
    resolution_ticket: u32,
}

impl Resource {
    pub fn new(path: &Path) -> Self {
        let metadata = path.metadata().ok();
        let changed = metadata
            .as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let internal_id = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let resource_type = if path.is_dir() {
            ResourceType::Folder
        } else if path
            .extension()
            .is_some_and(|e| e == "zip" || e == "jar" || e == "disabled")
        {
            ResourceType::ZipFile
        } else if path.extension().is_some_and(|e| e == "litemod") {
            ResourceType::Litemod
        } else {
            ResourceType::SingleFile
        };

        Resource {
            path: path.to_path_buf(),
            changed_date_time: changed,
            internal_id: internal_id.clone(),
            name: internal_id,
            resource_type,
            enabled: true,
            is_resolving: false,
            is_resolved: false,
            resolution_ticket: 0,
        }
    }

    pub fn with_name(path: &Path, name: &str) -> Self {
        let mut r = Resource::new(path);
        r.name = name.to_string();
        r
    }

    pub fn fileinfo(&self) -> &Path {
        &self.path
    }
    pub fn date_time_changed(&self) -> i64 {
        self.changed_date_time
    }
    pub fn internal_id(&self) -> &str {
        &self.internal_id
    }
    pub fn resource_type(&self) -> ResourceType {
        self.resource_type
    }
    pub fn enabled(&self) -> bool {
        self.enabled
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }
    pub fn valid(&self) -> bool {
        !matches!(self.resource_type, ResourceType::Unknown)
    }
    pub fn should_resolve(&self) -> bool {
        !self.is_resolving && !self.is_resolved
    }
    pub fn is_resolving(&self) -> bool {
        self.is_resolving
    }
    pub fn is_resolved(&self) -> bool {
        self.is_resolved
    }
    pub fn resolution_ticket(&self) -> u32 {
        self.resolution_ticket
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn enable(&mut self, action: EnableAction) -> bool {
        match action {
            EnableAction::Enable => {
                let old = self.enabled;
                self.enabled = true;
                old != self.enabled
            }
            EnableAction::Disable => {
                let old = self.enabled;
                self.enabled = false;
                old != self.enabled
            }
            EnableAction::Toggle => {
                self.enabled = !self.enabled;
                true
            }
        }
    }

    pub fn set_resolving(&mut self, resolving: bool, ticket: u32) {
        self.is_resolving = resolving;
        self.resolution_ticket = ticket;
    }

    pub fn set_resolved(&mut self, resolved: bool) {
        self.is_resolved = resolved;
    }

    pub fn destroy(&mut self) -> bool {
        if self.path.exists() {
            let result = if self.path.is_dir() {
                std::fs::remove_dir_all(&self.path)
            } else {
                std::fs::remove_file(&self.path)
            };
            result.is_ok()
        } else {
            false
        }
    }

    pub fn compare(&self, other: &Resource, sort_type: SortType) -> std::cmp::Ordering {
        match sort_type {
            SortType::Name => self.name.cmp(&other.name),
            SortType::Date => self.changed_date_time.cmp(&other.changed_date_time),
            SortType::Enabled => self.enabled.cmp(&other.enabled),
        }
    }

    pub fn apply_filter(&self, filter: &str) -> bool {
        if filter.is_empty() {
            return true;
        }
        let lower = filter.to_lowercase();
        self.name.to_lowercase().contains(&lower)
            || self.internal_id.to_lowercase().contains(&lower)
    }
}

#[derive(Debug, Clone)]
pub struct ModDetails {
    pub mod_id: String,
    pub name: String,
    pub version: String,
    pub mcversion: String,
    pub homeurl: String,
    pub description: String,
    pub authors: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModStatus {
    Installed,
    NotInstalled,
    NoMetadata,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Mod {
    pub resource: Resource,
    pub details: ModDetails,
    pub status: ModStatus,
}

impl Mod {
    pub fn new(path: &Path) -> Self {
        let mut m = Mod {
            resource: Resource::new(path),
            details: ModDetails {
                mod_id: String::new(),
                name: String::new(),
                version: String::new(),
                mcversion: String::new(),
                homeurl: String::new(),
                description: String::new(),
                authors: Vec::new(),
            },
            status: ModStatus::Unknown,
        };
        m.parse();
        m
    }

    pub fn name(&self) -> &str {
        if !self.details.name.is_empty() {
            &self.details.name
        } else {
            self.resource.name()
        }
    }

    pub fn version(&self) -> &str {
        &self.details.version
    }
    pub fn homeurl(&self) -> &str {
        &self.details.homeurl
    }
    pub fn description(&self) -> &str {
        &self.details.description
    }
    pub fn authors(&self) -> &[String] {
        &self.details.authors
    }

    pub fn parse(&mut self) {
        let path = self.resource.fileinfo().to_path_buf();
        if !path.exists() {
            return;
        }

        if path.is_dir() {
            self.parse_folder(&path);
        } else if path.is_file() {
            self.parse_zip(&path);
        }
    }

    fn parse_folder(&mut self, path: &Path) {
        // Check for fabric.mod.json
        let fabric_json = path.join("fabric.mod.json");
        if fabric_json.exists() {
            if let Ok(content) = std::fs::read_to_string(&fabric_json) {
                self.parse_fabric_json(&content);
                return;
            }
        }

        // Check for mods.toml (Forge)
        let mods_toml = path.join("META-INF").join("mods.toml");
        if mods_toml.exists() {
            if let Ok(content) = std::fs::read_to_string(&mods_toml) {
                self.parse_mods_toml(&content);
                return;
            }
        }

        // Check for META-INF/mcmod.info (legacy Forge)
        let mcmod_info = path.join("mcmod.info");
        if mcmod_info.exists() {
            if let Ok(content) = std::fs::read_to_string(&mcmod_info) {
                self.parse_mcmod_info(&content);
            }
        }
    }

    fn parse_zip(&mut self, path: &Path) {
        let file = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(_) => return,
        };
        let mut archive = match zip::ZipArchive::new(file) {
            Ok(a) => a,
            Err(_) => return,
        };

        // Try to find fabric.mod.json
        if let Ok(entry) = archive.by_name("fabric.mod.json") {
            if let Ok(content) = read_zip_entry(entry) {
                if self.parse_fabric_json(&content) {
                    return;
                }
            }
        }

        // Try META-INF/mods.toml
        if let Ok(entry) = archive.by_name("META-INF/mods.toml") {
            if let Ok(content) = read_zip_entry(entry) {
                if self.parse_mods_toml(&content) {
                    return;
                }
            }
        }

        // Try mcmod.info
        for i in 0..archive.len() {
            if let Ok(entry) = archive.by_index(i) {
                let name = entry.name().to_string();
                if name == "mcmod.info" || name.ends_with("/mcmod.info") {
                    if let Ok(content) = read_zip_entry(entry) {
                        if self.parse_mcmod_info(&content) {
                            return;
                        }
                    }
                }
            }
        }

        // Try META-INF/MANIFEST.MF for basic info
        if let Ok(mut entry) = archive.by_name("META-INF/MANIFEST.MF") {
            let mut content = String::new();
            use std::io::Read;
            if entry.read_to_string(&mut content).is_ok() {
                self.parse_manifest(&content);
            }
        };
    }

    fn parse_fabric_json(&mut self, content: &str) -> bool {
        let json: serde_json::Value = match serde_json::from_str(content) {
            Ok(v) => v,
            Err(_) => return false,
        };

        if let Some(id) = json.get("id").and_then(|v| v.as_str()) {
            self.details.mod_id = id.to_string();
        }
        if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
            self.details.name = name.to_string();
        }
        if let Some(ver) = json.get("version").and_then(|v| v.as_str()) {
            self.details.version = ver.to_string();
        }
        if let Some(desc) = json.get("description").and_then(|v| v.as_str()) {
            self.details.description = desc.to_string();
        }
        if let Some(authors) = json.get("authors").and_then(|v| v.as_array()) {
            for author in authors {
                if let Some(a) = author.as_str() {
                    self.details.authors.push(a.to_string());
                } else if let Some(obj) = author.as_object() {
                    if let Some(name) = obj.get("name").and_then(|v| v.as_str()) {
                        self.details.authors.push(name.to_string());
                    }
                }
            }
        }
        if let Some(contact) = json.get("contact").and_then(|v| v.as_object()) {
            if let Some(homepage) = contact.get("homepage").and_then(|v| v.as_str()) {
                self.details.homeurl = homepage.to_string();
            } else if let Some(sources) = contact.get("sources").and_then(|v| v.as_str()) {
                self.details.homeurl = sources.to_string();
            }
        }

        let has_id = !self.details.mod_id.is_empty();
        if has_id && self.details.name.is_empty() {
            self.details.name = self.details.mod_id.clone();
        }
        has_id
    }

    fn parse_mods_toml(&mut self, content: &str) -> bool {
        // Basic TOML parsing without a full TOML parser
        let mut in_mods = false;
        let mut found = false;

        for line in content.lines() {
            let line = line.trim();
            if line == "[[mods]]" {
                in_mods = true;
                continue;
            }
            if in_mods && line.starts_with('[') {
                if found {
                    break;
                }
                in_mods = false;
                continue;
            }
            if !in_mods {
                continue;
            }

            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim();
                let value = line[eq_pos + 1..].trim().trim_matches('"');
                match key {
                    "modId" => {
                        self.details.mod_id = value.to_string();
                        found = true;
                    }
                    "displayName" => {
                        self.details.name = value.to_string();
                    }
                    "version" => {
                        self.details.version = value.to_string();
                    }
                    "description" => {
                        self.details.description = value.to_string();
                    }
                    "displayURL" => {
                        self.details.homeurl = value.to_string();
                    }
                    "authors" => {
                        // Could be a list, but TOML parsing is minimal here
                        for author in value.split(',') {
                            self.details.authors.push(author.trim().to_string());
                        }
                    }
                    _ => {}
                }
            }
        }

        if self.details.name.is_empty() {
            self.details.name = self.details.mod_id.clone();
        }

        found
    }

    fn parse_mcmod_info(&mut self, content: &str) -> bool {
        let json: serde_json::Value = match serde_json::from_str(content) {
            Ok(v) => v,
            Err(_) => return false,
        };

        let arr = match json.as_array() {
            Some(a) => a,
            None => return false,
        };

        if let Some(first) = arr.first().and_then(|v| v.as_object()) {
            if let Some(modid) = first.get("modid").and_then(|v| v.as_str()) {
                self.details.mod_id = modid.to_string();
            }
            if let Some(name) = first.get("name").and_then(|v| v.as_str()) {
                self.details.name = name.to_string();
            }
            if let Some(ver) = first.get("version").and_then(|v| v.as_str()) {
                self.details.version = ver.to_string();
            }
            if let Some(desc) = first.get("description").and_then(|v| v.as_str()) {
                self.details.description = desc.to_string();
            }
            if let Some(url) = first.get("url").and_then(|v| v.as_str()) {
                self.details.homeurl = url.to_string();
            }
            if let Some(authors) = first.get("authors").and_then(|v| v.as_array()) {
                for author in authors {
                    if let Some(a) = author.as_str() {
                        self.details.authors.push(a.to_string());
                    }
                }
            } else if let Some(author_list) = first.get("authorList").and_then(|v| v.as_array()) {
                for author in author_list {
                    if let Some(a) = author.as_str() {
                        self.details.authors.push(a.to_string());
                    }
                }
            }
        }

        !self.details.mod_id.is_empty()
    }

    fn parse_manifest(&mut self, content: &str) {
        for line in content.lines() {
            if let Some(eq_pos) = line.find(':') {
                let key = line[..eq_pos].trim();
                let value = line[eq_pos + 1..].trim();
                match key {
                    "Implementation-Title" if self.details.name.is_empty() => {
                        self.details.name = value.to_string();
                    }
                    "Implementation-Version" if self.details.version.is_empty() => {
                        self.details.version = value.to_string();
                    }
                    _ => {}
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResourcePack {
    pub resource: Resource,
    pub pack_format: i32,
    pub description: String,
}

impl ResourcePack {
    pub fn new(path: &Path) -> Self {
        let mut rp = ResourcePack {
            resource: Resource::new(path),
            pack_format: 0,
            description: String::new(),
        };
        rp.parse();
        rp
    }

    pub fn parse(&mut self) {
        let path = self.resource.fileinfo().to_path_buf();
        if !path.exists() {
            return;
        }

        let content = if path.is_dir() {
            let mcmeta = path.join("pack.mcmeta");
            std::fs::read_to_string(&mcmeta).ok()
        } else {
            let file = std::fs::File::open(&path).ok();
            let mut archive = file.and_then(|f| zip::ZipArchive::new(f).ok());
            archive
                .as_mut()
                .and_then(|a| a.by_name("pack.mcmeta").ok())
                .and_then(|mut e| {
                    let mut s = String::new();
                    use std::io::Read;
                    e.read_to_string(&mut s).ok().map(|_| s)
                })
        };

        if let Some(content) = content {
            self.parse_mcmeta(&content);
        }
    }

    fn parse_mcmeta(&mut self, content: &str) {
        let json: serde_json::Value = match serde_json::from_str(content) {
            Ok(v) => v,
            Err(_) => return,
        };

        if let Some(pack) = json.get("pack").and_then(|v| v.as_object()) {
            if let Some(format) = pack.get("pack_format").and_then(|v| v.as_i64()) {
                self.pack_format = format as i32;
            }
            if let Some(desc) = pack.get("description").and_then(|v| v.as_str()) {
                self.description = desc.to_string();
            } else if let Some(desc_obj) = pack.get("description").and_then(|v| v.as_object()) {
                if let Some(text) = desc_obj.get("text").and_then(|v| v.as_str()) {
                    self.description = text.to_string();
                }
            }
        }

        if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
            self.resource.set_name(name);
        }
    }

    pub fn compatible_versions(&self) -> (String, String) {
        match self.pack_format {
            1..=3 => ("1.6.1".to_string(), "1.8.9".to_string()),
            4 => ("1.9".to_string(), "1.10.2".to_string()),
            5..=6 => ("1.11".to_string(), "1.12.2".to_string()),
            7..=8 => ("1.13".to_string(), "1.14.4".to_string()),
            9..=10 => ("1.15".to_string(), "1.16.1".to_string()),
            11..=12 => ("1.16.2".to_string(), "1.16.5".to_string()),
            13..=14 => ("1.17".to_string(), "1.17.1".to_string()),
            15..=16 => ("1.18".to_string(), "1.18.2".to_string()),
            17..=18 => ("1.19".to_string(), "1.19.2".to_string()),
            19 => ("1.19.3".to_string(), "1.19.3".to_string()),
            20 => ("1.19.4".to_string(), "1.19.4".to_string()),
            21 => ("1.20".to_string(), "1.20.1".to_string()),
            22 => ("1.20.2".to_string(), "1.20.2".to_string()),
            23 => ("1.20.3".to_string(), "1.20.4".to_string()),
            24 => ("1.20.5".to_string(), "1.20.6".to_string()),
            _ => ("unknown".to_string(), "unknown".to_string()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TexturePack {
    pub resource: Resource,
    pub description: String,
}

impl TexturePack {
    pub fn new(path: &Path) -> Self {
        let mut tp = TexturePack {
            resource: Resource::new(path),
            description: String::new(),
        };
        tp.parse();
        tp
    }

    pub fn parse(&mut self) {
        let path = self.resource.fileinfo().to_path_buf();
        if !path.exists() {
            return;
        }

        let content = if path.is_dir() {
            let pack_txt = path.join("pack.txt");
            std::fs::read_to_string(&pack_txt).ok()
        } else {
            let file = std::fs::File::open(&path).ok();
            let mut archive = file.and_then(|f| zip::ZipArchive::new(f).ok());
            archive
                .as_mut()
                .and_then(|a| a.by_name("pack.txt").ok())
                .and_then(|mut e| {
                    let mut s = String::new();
                    use std::io::Read;
                    e.read_to_string(&mut s).ok().map(|_| s)
                })
        };

        if let Some(content) = content {
            self.description = content.trim().to_string();
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResourceFolderModel<T> {
    resources: Vec<T>,
    dir_path: PathBuf,
}

impl<T> ResourceFolderModel<T> {
    pub fn new(dir: &Path) -> Self {
        ResourceFolderModel {
            resources: Vec::new(),
            dir_path: dir.to_path_buf(),
        }
    }

    pub fn size(&self) -> usize {
        self.resources.len()
    }
    pub fn is_empty(&self) -> bool {
        self.resources.is_empty()
    }
    pub fn all(&self) -> &[T] {
        &self.resources
    }
    pub fn at(&self, index: usize) -> Option<&T> {
        self.resources.get(index)
    }
    pub fn at_mut(&mut self, index: usize) -> Option<&mut T> {
        self.resources.get_mut(index)
    }
    pub fn dir(&self) -> &Path {
        &self.dir_path
    }

    pub fn add(&mut self, resource: T) {
        self.resources.push(resource);
    }

    pub fn remove(&mut self, index: usize) -> Option<T> {
        if index < self.resources.len() {
            Some(self.resources.remove(index))
        } else {
            None
        }
    }

    pub fn clear(&mut self) {
        self.resources.clear();
    }
}

impl ResourceFolderModel<Mod> {
    pub fn load_mods(&mut self) {
        self.resources.clear();
        if !self.dir_path.exists() {
            return;
        }

        let mut entries: Vec<_> = std::fs::read_dir(&self.dir_path)
            .into_iter()
            .flatten()
            .flatten()
            .collect();

        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext == "jar" || ext == "zip" || ext == "disabled" || path.is_dir() {
                let m = Mod::new(&path);
                if m.resource.valid() {
                    self.resources.push(m);
                }
            }
        }
    }
}

impl ResourceFolderModel<ResourcePack> {
    pub fn load_resource_packs(&mut self) {
        self.resources.clear();
        if !self.dir_path.exists() {
            return;
        }

        let mut entries: Vec<_> = std::fs::read_dir(&self.dir_path)
            .into_iter()
            .flatten()
            .flatten()
            .collect();

        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            if path
                .extension()
                .is_none_or(|e| e != "zip" && e != "disabled")
                && !path.is_dir()
            {
                continue;
            }
            let rp = ResourcePack::new(&path);
            if rp.resource.valid() {
                self.resources.push(rp);
            }
        }
    }
}

impl ResourceFolderModel<TexturePack> {
    pub fn load_texture_packs(&mut self) {
        self.resources.clear();
        if !self.dir_path.exists() {
            return;
        }

        let mut entries: Vec<_> = std::fs::read_dir(&self.dir_path)
            .into_iter()
            .flatten()
            .flatten()
            .collect();

        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let path = entry.path();
            if path
                .extension()
                .is_none_or(|e| e != "zip" && e != "disabled")
                && !path.is_dir()
            {
                continue;
            }
            let tp = TexturePack::new(&path);
            if tp.resource.valid() {
                self.resources.push(tp);
            }
        }
    }
}

fn read_zip_entry<R: std::io::Read>(mut entry: R) -> Result<String, String> {
    let mut content = String::new();
    entry
        .read_to_string(&mut content)
        .map_err(|e| e.to_string())?;
    Ok(content)
}
