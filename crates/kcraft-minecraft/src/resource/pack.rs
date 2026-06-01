use std::path::Path;

use super::resource_item::Resource;

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
