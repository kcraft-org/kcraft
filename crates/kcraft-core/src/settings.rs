use std::collections::HashMap;
use std::fmt;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    Invalid,
}

impl Value {
    pub fn is_valid(&self) -> bool {
        !matches!(self, Value::Invalid)
    }

    pub fn to_bool(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::String(s) => !s.is_empty() && s != "false" && s != "0",
            Value::Int(i) => *i != 0,
            _ => false,
        }
    }

    pub fn to_string_value(&self) -> String {
        self.to_string()
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Int(i) => write!(f, "{}", i),
            Value::Float(fl) => write!(f, "{}", fl),
            Value::Invalid => write!(f, ""),
        }
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}
impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}
impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}
impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Int(i)
    }
}
impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Value::Int(i as i64)
    }
}

#[derive(Debug)]
pub struct INIFile {
    data: HashMap<String, String>,
}

impl INIFile {
    pub fn new() -> Self {
        INIFile {
            data: HashMap::new(),
        }
    }

    pub fn load(&mut self, content: &str) {
        self.data.clear();
        for line in content.lines() {
            let line = Self::strip_comment(line).trim();
            if line.is_empty() {
                continue;
            }
            if let Some(eq_pos) = line.find('=') {
                let key = line[..eq_pos].trim();
                let value = Self::unescape(line[eq_pos + 1..].trim());
                if !key.is_empty() {
                    self.data.insert(key.to_string(), value);
                }
            }
        }
    }

    pub fn load_file(&mut self, path: &Path) -> std::io::Result<()> {
        let content = std::fs::read_to_string(path)?;
        self.load(&content);
        Ok(())
    }

    pub fn save_file(&self, path: &Path) -> std::io::Result<()> {
        let mut content = String::new();
        let mut keys: Vec<&String> = self.data.keys().collect();
        keys.sort();
        for key in keys {
            if let Some(value) = self.data.get(key) {
                content.push_str(&format!("{}={}\n", key, Self::escape(value)));
            }
        }
        std::fs::write(path, content.as_bytes())?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.data.get(key).map(|s| s.as_str())
    }
    pub fn set(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }
    pub fn remove(&mut self, key: &str) {
        self.data.remove(key);
    }

    fn strip_comment(line: &str) -> &str {
        let mut in_escape = false;
        for (i, c) in line.char_indices() {
            if in_escape {
                in_escape = false;
                continue;
            }
            if c == '\\' {
                in_escape = true;
                continue;
            }
            if c == '#' {
                return &line[..i];
            }
        }
        line
    }

    fn unescape(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        let mut chars = s.chars();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => result.push('\n'),
                    Some('t') => result.push('\t'),
                    Some('\\') => result.push('\\'),
                    Some('#') => result.push('#'),
                    Some(c) => {
                        result.push('\\');
                        result.push(c);
                    }
                    None => result.push('\\'),
                }
            } else {
                result.push(c);
            }
        }
        result
    }

    fn escape(s: &str) -> String {
        let mut result = String::with_capacity(s.len());
        for c in s.chars() {
            match c {
                '\n' => result.push_str("\\n"),
                '\t' => result.push_str("\\t"),
                '\\' => result.push_str("\\\\"),
                '#' => result.push_str("\\#"),
                c => result.push(c),
            }
        }
        result
    }
}

impl Default for INIFile {
    fn default() -> Self {
        Self::new()
    }
}

type IniHandle = Arc<Mutex<IniState>>;

#[derive(Clone)]
pub struct INISettingsObject {
    ini: IniHandle,
    file_path: std::path::PathBuf,
}

struct IniState {
    data: HashMap<String, String>,
    suspend_save: bool,
    do_save: bool,
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

    fn flush(state: &IniState) {
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

#[derive(Clone)]
pub struct Setting {
    synonyms: Vec<String>,
    def_val: Value,
    ini: Option<IniHandle>,
    keys: Vec<String>,
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

pub struct SettingsLock {
    obj: INISettingsObject,
}

impl SettingsLock {
    pub fn new(obj: &INISettingsObject) -> Self {
        obj.suspend_save();
        SettingsLock { obj: obj.clone() }
    }
}

impl Drop for SettingsLock {
    fn drop(&mut self) {
        self.obj.resume_save();
    }
}
