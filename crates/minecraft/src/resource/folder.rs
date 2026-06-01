use std::path::{Path, PathBuf};

use super::mod_model::Mod;
use super::pack::ResourcePack;
use super::texture::TexturePack;

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
