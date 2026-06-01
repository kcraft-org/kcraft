use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::sync::{Arc, Mutex};

use super::ini::INIFile;

pub(crate) type IniHandle = Arc<Mutex<IniState>>;

#[derive(Clone)]
pub struct INISettingsObject {
    ini: IniHandle,
    file_path: std::path::PathBuf,
}

pub(crate) struct IniState {
    pub(crate) data: HashMap<String, String>,
    pub(crate) suspend_save: bool,
    pub(crate) do_save: bool,
    file_path: std::path::PathBuf,
}

impl fmt::Debug for INISettingsObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("INISettingsObject")
            .field("file_path", &self.file_path)
            .finish()
    }
}

impl INISettingsObject {
    pub fn new(path: &Path) -> Self {
        let mut ini = INIFile::new();
        let _ = ini.load_file(path);

        let state = Arc::new(Mutex::new(IniState {
            data: ini.data,
            suspend_save: false,
            do_save: false,
            file_path: path.to_path_buf(),
        }));

        INISettingsObject {
            ini: state.clone(),
            file_path: path.to_path_buf(),
        }
    }

    pub fn register_setting(&self, synonyms: Vec<String>, def_val: Value) -> Setting {
        let ini = self.ini.clone();
        let keys = synonyms.clone();

        Setting {
            synonyms,
            def_val,
            ini: Some(ini),
            keys,
        }
    }

    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    pub fn reload(&self) {
        let mut state = self.ini.lock().unwrap();
        let mut ini = INIFile {
            data: std::mem::take(&mut state.data),
        };
        let _ = ini.load_file(&state.file_path);
        state.data = ini.data;
    }

    pub fn suspend_save(&self) {
        let mut state = self.ini.lock().unwrap();
        state.suspend_save = true;
    }

    pub fn resume_save(&self) {
        let mut state = self.ini.lock().unwrap();
        state.suspend_save = false;
        if state.do_save {
            Self::flush(&state);
            state.do_save = false;
        }
    }

    pub(crate) fn flush(state: &IniState) {
        let ini = INIFile {
            data: state.data.clone(),
        };
        let _ = ini.save_file(&state.file_path);
    }

    pub fn get_raw(&self, key: &str) -> Option<String> {
        let state = self.ini.lock().unwrap();
        state.data.get(key).cloned()
    }

    pub fn set_raw(&self, key: &str, value: &str) {
        let mut state = self.ini.lock().unwrap();
        state.data.insert(key.to_string(), value.to_string());
        if state.suspend_save {
            state.do_save = true;
        } else {
            Self::flush(&state);
        }
    }

    pub fn remove_raw(&self, key: &str) {
        let mut state = self.ini.lock().unwrap();
        state.data.remove(key);
        if state.suspend_save {
            state.do_save = true;
        } else {
            Self::flush(&state);
        }
    }
}

use super::setting::Setting;
use super::value::Value;
