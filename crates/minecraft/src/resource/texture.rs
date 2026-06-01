use std::path::Path;

use super::resource_item::Resource;

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
