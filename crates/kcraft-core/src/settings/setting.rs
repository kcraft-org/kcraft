use std::fmt;

use super::settings_object::{INISettingsObject, IniHandle};
use super::value::Value;

#[derive(Clone)]
pub struct Setting {
    pub(crate) synonyms: Vec<String>,
    pub(crate) def_val: Value,
    pub(crate) ini: Option<IniHandle>,
    pub(crate) keys: Vec<String>,
}

impl fmt::Debug for Setting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Setting")
            .field("id", &self.synonyms.first())
            .field("def_val", &self.def_val)
            .finish()
    }
}

impl Setting {
    pub fn new(synonyms: Vec<String>, def_val: Value) -> Self {
        Setting {
            synonyms: synonyms.clone(),
            def_val,
            ini: None,
            keys: synonyms,
        }
    }

    pub fn id(&self) -> &str {
        self.synonyms.first().map(|s| s.as_str()).unwrap_or("")
    }

    pub fn config_keys(&self) -> &[String] {
        &self.synonyms
    }

    pub fn def_value(&self) -> Value {
        self.def_val.clone()
    }

    pub fn get(&self) -> Value {
        match &self.ini {
            Some(ini) => {
                let state = ini.lock().unwrap();
                for key in &self.keys {
                    if let Some(val) = state.data.get(key) {
                        return Value::String(val.clone());
                    }
                }
                self.def_val.clone()
            }
            None => self.def_val.clone(),
        }
    }

    pub fn set(&self, value: Value) {
        if let Some(ini) = &self.ini {
            let mut state = ini.lock().unwrap();
            if value.is_valid() {
                if let Some(first) = self.keys.first() {
                    state.data.insert(first.clone(), value.to_string());
                }
                for key in self.keys.iter().skip(1) {
                    state.data.remove(key);
                }
            } else {
                for key in &self.keys {
                    state.data.remove(key);
                }
            }
            if state.suspend_save {
                state.do_save = true;
            } else {
                INISettingsObject::flush(&state);
            }
        }
    }

    pub fn reset(&self) {
        if let Some(ini) = &self.ini {
            let mut state = ini.lock().unwrap();
            for key in &self.keys {
                state.data.remove(key);
            }
            if state.suspend_save {
                state.do_save = true;
            } else {
                INISettingsObject::flush(&state);
            }
        }
    }
}
