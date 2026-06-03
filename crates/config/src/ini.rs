use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct IniFile {
    values: HashMap<String, String>,
}

impl IniFile {
    pub fn new() -> Self {
        IniFile {
            values: HashMap::new(),
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self, String> {
        let data = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read INI file: {}", e))?;
        Self::parse(&data)
    }

    pub fn parse(data: &str) -> Result<Self, String> {
        let mut values = HashMap::new();
        for line in data.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let no_comment = strip_comment(trimmed);
            let trimmed = no_comment.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(eq_pos) = trimmed.find('=') {
                let key = trimmed[..eq_pos].trim().to_string();
                let value = trimmed[eq_pos + 1..].trim().to_string();
                let unescaped = unescape(&value);
                values.insert(key, unescaped);
            }
        }
        Ok(IniFile { values })
    }

    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), String> {
        let mut data = String::new();
        let mut keys: Vec<&String> = self.values.keys().collect();
        keys.sort();
        for key in keys {
            if let Some(value) = self.values.get(key) {
                let escaped = escape(value);
                data.push_str(&format!("{}={}\n", key, escaped));
            }
        }
        fs::write(path, data.as_bytes()).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(|s| s.as_str())
    }

    pub fn get_or_default(&self, key: &str, default: &str) -> String {
        self.values
            .get(key)
            .map(|s| s.as_str())
            .unwrap_or(default)
            .to_string()
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    pub fn remove(&mut self, key: &str) {
        self.values.remove(key);
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn raw_values(&self) -> &HashMap<String, String> {
        &self.values
    }
}

impl Default for IniFile {
    fn default() -> Self {
        Self::new()
    }
}

fn strip_comment(line: &str) -> &str {
    let mut in_escape = false;
    for (i, ch) in line.char_indices() {
        if in_escape {
            in_escape = false;
            continue;
        }
        if ch == '\\' {
            in_escape = true;
            continue;
        }
        if ch == '#' {
            return &line[..i];
        }
    }
    line
}

fn unescape(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
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
            result.push(ch);
        }
    }
    result
}

fn escape(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\n' => result.push_str("\\n"),
            '\t' => result.push_str("\\t"),
            '\\' => result.push_str("\\\\"),
            '#' => result.push_str("\\#"),
            c => result.push(c),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let data = "key1=value1\nkey2=value2\n";
        let ini = IniFile::parse(data).unwrap();
        assert_eq!(ini.get("key1"), Some("value1"));
        assert_eq!(ini.get("key2"), Some("value2"));
    }

    #[test]
    fn test_parse_with_comments() {
        let data = "# comment\nkey1=value1\n# another\nkey2=value2\n";
        let ini = IniFile::parse(data).unwrap();
        assert_eq!(ini.get("key1"), Some("value1"));
        assert_eq!(ini.get("key2"), Some("value2"));
    }

    #[test]
    fn test_parse_escaped_hash() {
        let data = "key1=value\\#1\nkey2=value2\n";
        let ini = IniFile::parse(data).unwrap();
        assert_eq!(ini.get("key1"), Some("value#1"));
        assert_eq!(ini.get("key2"), Some("value2"));
    }

    #[test]
    fn test_escape_roundtrip() {
        let mut ini = IniFile::new();
        ini.set("key1", "hello\nworld");
        ini.set("key2", "tab\there");
        ini.set("key3", "hash#sign");
        let serialized = {
            let mut data = String::new();
            let mut keys: Vec<&String> = ini.values.keys().collect();
            keys.sort();
            for key in keys {
                if let Some(value) = ini.values.get(key) {
                    let escaped = escape(value);
                    data.push_str(&format!("{}={}\n", key, escaped));
                }
            }
            data
        };
        let parsed = IniFile::parse(&serialized).unwrap();
        assert_eq!(parsed.get("key1"), Some("hello\nworld"));
        assert_eq!(parsed.get("key2"), Some("tab\there"));
        assert_eq!(parsed.get("key3"), Some("hash#sign"));
    }

    #[test]
    fn test_ini_save_load_roundtrip() {
        let dir = std::env::temp_dir().join("kcraft_test_ini_roundtrip");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("test.ini");

        let mut ini = IniFile::new();
        ini.set("key1", "value1");
        ini.set("key2", "value with spaces");
        ini.set("key3", "value=with=equals");
        ini.save(&path).unwrap();

        let loaded = IniFile::load(&path).unwrap();
        assert_eq!(loaded.get("key1").unwrap(), "value1");
        assert_eq!(loaded.get("key2").unwrap(), "value with spaces");
        assert_eq!(loaded.get("key3").unwrap(), "value=with=equals");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_ini_section_handling() {
        let mut ini = IniFile::new();
        ini.set("InstanceType", "OneSix");
        ini.set("name", "Test Instance");
        ini.set("iconKey", "default");
        assert_eq!(ini.get("InstanceType").unwrap(), "OneSix");
        assert_eq!(ini.get("name").unwrap(), "Test Instance");
        assert_eq!(ini.get("iconKey").unwrap(), "default");
        assert!(ini.get("nonexistent").is_none());
    }
}
