use std::path::{Path, PathBuf};

use super::model::World;

#[derive(Debug, Clone)]
pub struct WorldList {
    worlds: Vec<World>,
    dir_path: PathBuf,
}

impl WorldList {
    pub fn new(dir: &Path) -> Self {
        let mut wl = WorldList {
            worlds: Vec::new(),
            dir_path: dir.to_path_buf(),
        };
        wl.load();
        wl
    }

    pub fn worlds(&self) -> &[World] {
        &self.worlds
    }

    pub fn load(&mut self) {
        self.worlds.clear();

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
            let world = World::new(&path);
            if world.is_valid() {
                self.worlds.push(world);
            }
        }
    }

    pub fn reload(&mut self) {
        self.load();
    }

    pub fn get_world(&self, index: usize) -> Option<&World> {
        self.worlds.get(index)
    }

    pub fn get_world_by_name(&self, name: &str) -> Option<&World> {
        self.worlds.iter().find(|w| w.folder_name() == name)
    }

    pub fn delete_world(&mut self, index: usize) -> bool {
        if index < self.worlds.len() {
            self.worlds[index].destroy();
            self.worlds.remove(index);
            true
        } else {
            false
        }
    }

    pub fn len(&self) -> usize {
        self.worlds.len()
    }
    pub fn is_empty(&self) -> bool {
        self.worlds.is_empty()
    }
}
