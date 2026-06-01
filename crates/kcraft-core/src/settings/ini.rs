use std::collections::HashMap;
use std::path::Path;

#[derive(Debug)]
pub struct INIFile {
    pub(crate) data: HashMap<String, String>,
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
