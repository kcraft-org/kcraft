use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;
use tracing::warn;

use crate::ini::IniFile;
use crate::setting::Setting;
use crate::{ConfigError, Result};

pub trait SettingsStorage: Send + Sync {
    fn load(&mut self) -> Result<HashMap<String, String>>;
    fn save(&self, values: &HashMap<String, String>) -> Result<()>;
}

pub struct IniSettingsStorage {
    path: PathBuf,
}

impl IniSettingsStorage {
    pub fn new(path: PathBuf) -> Self {
        IniSettingsStorage { path }
    }
}

impl SettingsStorage for IniSettingsStorage {
    fn load(&mut self) -> Result<HashMap<String, String>> {
        if !self.path.exists() {
            return Ok(HashMap::new());
        }
        let ini = IniFile::load(&self.path)
            .map_err(ConfigError::Parse)?;
        Ok(ini.raw_values().clone())
    }

    fn save(&self, values: &HashMap<String, String>) -> Result<()> {
        let mut ini = IniFile::new();
        for (k, v) in values {
            ini.set(k, v);
        }
        ini.save(&self.path)
            .map_err(ConfigError::Parse)?;
        Ok(())
    }
}

pub struct SettingsObject {
    storage: Box<dyn SettingsStorage>,
    settings: RwLock<HashMap<String, Setting>>,
    raw_values: RwLock<HashMap<String, String>>,
    suspend_count: RwLock<u32>,
    pending_save: RwLock<bool>,
}

impl SettingsObject {
    pub fn new(mut storage: Box<dyn SettingsStorage>) -> Result<Self> {
        let mut raw_values = HashMap::new();
        let loaded = storage.load()?;
        raw_values.extend(loaded);

        Ok(SettingsObject {
            storage,
            settings: RwLock::new(HashMap::new()),
            raw_values: RwLock::new(raw_values),
            suspend_count: RwLock::new(0),
            pending_save: RwLock::new(false),
        })
    }

    pub fn new_ini(path: PathBuf) -> Result<Self> {
        Self::new(Box::new(IniSettingsStorage::new(path)))
    }

    pub fn register<T: crate::SettingValue + 'static>(
        &self,
        synonyms: Vec<String>,
        default_value: T,
    ) -> Setting {
        let setting = Setting::new(synonyms.clone(), default_value);
        let id = setting.id().to_string();
        self.settings.write().unwrap().insert(id.clone(), setting.clone());
        setting
    }

    pub fn register_simple<T: crate::SettingValue + 'static>(
        &self,
        id: &str,
        default_value: T,
    ) -> Setting {
        self.register(vec![id.to_string()], default_value)
    }

    pub fn get_setting(&self, id: &str) -> Option<Setting> {
        self.settings.read().unwrap().get(id).cloned()
    }

    pub fn get<T: crate::SettingValue + 'static + Clone>(&self, id: &str) -> Result<T> {
        let raw = self.raw_values.read().unwrap();
        let setting = self.settings.read().unwrap();
        let s = setting.get(id).ok_or_else(|| ConfigError::SettingNotFound(id.to_string()))?;

        for key in s.config_keys() {
            if let Some(val) = raw.get(key) {
                return parse_value::<T>(val);
            }
        }

        if let Some(default_val) = s.default_value().as_any().downcast_ref::<T>() {
            return Ok(default_val.clone());
        }
        Err(ConfigError::SettingNotFound(id.to_string()))
    }

    pub fn set(&self, id: &str, value: &str) -> Result<()> {
        let setting = self.settings.read().unwrap();
        let s = setting.get(id).ok_or_else(|| ConfigError::SettingNotFound(id.to_string()))?;
        let key = s.config_keys().first().cloned().unwrap_or_else(|| id.to_string());
        drop(setting);

        self.raw_values.write().unwrap().insert(key.clone(), value.to_string());
        self.maybe_save();
        Ok(())
    }

    pub fn contains(&self, id: &str) -> bool {
        self.settings.read().unwrap().contains_key(id)
    }

    pub fn reset(&self, id: &str) -> Result<()> {
        let setting = self.settings.read().unwrap();
        let s = setting.get(id).ok_or_else(|| ConfigError::SettingNotFound(id.to_string()))?;
        for key in s.config_keys() {
            self.raw_values.write().unwrap().remove(key);
        }
        drop(setting);
        self.maybe_save();
        Ok(())
    }

    pub fn reload(&mut self) -> Result<()> {
        let loaded = self.storage.load()?;
        let mut raw = self.raw_values.write().unwrap();
        raw.clear();
        raw.extend(loaded);
        Ok(())
    }

    pub fn suspend_save(&self) {
        *self.suspend_count.write().unwrap() += 1;
    }

    pub fn resume_save(&self) {
        let mut count = self.suspend_count.write().unwrap();
        if *count > 0 {
            *count -= 1;
        }
        if *count == 0 {
            let has_pending = *self.pending_save.read().unwrap();
            if has_pending {
                self.flush_save();
            }
        }
    }

    fn maybe_save(&self) {
        let count = *self.suspend_count.read().unwrap();
        if count > 0 {
            *self.pending_save.write().unwrap() = true;
            return;
        }
        self.flush_save();
    }

    fn flush_save(&self) {
        let raw = self.raw_values.read().unwrap();
        if let Err(e) = self.storage.save(&raw) {
            warn!("Failed to save settings: {}", e);
        }
        *self.pending_save.write().unwrap() = false;
    }
}

fn parse_value<T: crate::SettingValue + 'static>(val: &str) -> Result<T> {
    let value: Box<dyn std::any::Any> = if std::any::TypeId::of::<T>() == std::any::TypeId::of::<String>() {
        Box::new(val.to_string())
    } else if std::any::TypeId::of::<T>() == std::any::TypeId::of::<bool>() {
        Box::new(val.parse::<bool>().unwrap_or(false))
    } else if std::any::TypeId::of::<T>() == std::any::TypeId::of::<i32>() {
        Box::new(val.parse::<i32>().unwrap_or(0))
    } else if std::any::TypeId::of::<T>() == std::any::TypeId::of::<i64>() {
        Box::new(val.parse::<i64>().unwrap_or(0))
    } else if std::any::TypeId::of::<T>() == std::any::TypeId::of::<f64>() {
        Box::new(val.parse::<f64>().unwrap_or(0.0))
    } else {
        return Err(ConfigError::TypeMismatch(std::any::type_name::<T>().to_string()));
    };
    let boxed: Box<dyn std::any::Any> = value;
    let result = boxed.downcast::<T>().map(|b| *b);
    result.map_err(|_| ConfigError::TypeMismatch(std::any::type_name::<T>().to_string()))
}
