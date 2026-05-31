use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[derive(Default)]
pub enum GameType {
    Unknown = -1,
    #[default]
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    Spectator = 3,
}

impl GameType {
    pub fn from_int(value: i32) -> Self {
        match value {
            0 => GameType::Survival,
            1 => GameType::Creative,
            2 => GameType::Adventure,
            3 => GameType::Spectator,
            _ => GameType::Unknown,
        }
    }
}


#[derive(Debug, Clone)]
pub struct World {
    folder_name: String,
    actual_name: String,
    icon_file: String,
    size: i64,
    last_played: i64,
    random_seed: i64,
    game_type: GameType,
    container_path: PathBuf,
    is_valid: bool,
}

impl World {
    pub fn new(path: &Path) -> Self {
        let mut world = World {
            folder_name: String::new(),
            actual_name: String::new(),
            icon_file: String::new(),
            size: 0,
            last_played: 0,
            random_seed: 0,
            game_type: GameType::Unknown,
            container_path: path.to_path_buf(),
            is_valid: false,
        };

        if path.is_dir() {
            world.read_from_fs(path);
        } else if path.is_file() && path.extension().is_some_and(|e| e == "zip") {
            world.read_from_zip(path);
        }

        world
    }

    pub fn folder_name(&self) -> &str { &self.folder_name }
    pub fn name(&self) -> &str { &self.actual_name }
    pub fn icon_file(&self) -> &str { &self.icon_file }
    pub fn bytes(&self) -> i64 { self.size }
    pub fn last_played(&self) -> i64 { self.last_played }
    pub fn game_type(&self) -> GameType { self.game_type }
    pub fn seed(&self) -> i64 { self.random_seed }
    pub fn is_valid(&self) -> bool { self.is_valid }
    pub fn is_on_fs(&self) -> bool { self.container_path.is_dir() }
    pub fn container(&self) -> &Path { &self.container_path }

    pub fn destroy(&mut self) -> bool {
        if self.container_path.exists() {
            let result = if self.container_path.is_dir() {
                std::fs::remove_dir_all(&self.container_path)
            } else {
                std::fs::remove_file(&self.container_path)
            };
            result.is_ok()
        } else {
            false
        }
    }

    pub fn rename(&mut self, to: &str) -> bool {
        let new_path = self.container_path.parent().unwrap_or(Path::new(".")).join(to);
        if std::fs::rename(&self.container_path, &new_path).is_ok() {
            self.container_path = new_path;
            self.folder_name = to.to_string();
            true
        } else {
            false
        }
    }

    pub fn reset_icon(&mut self) -> bool {
        if !self.icon_file.is_empty() {
            let icon_path = Path::new(&self.icon_file);
            if icon_path.exists() {
                return std::fs::remove_file(icon_path).is_ok();
            }
        }
        false
    }

    fn read_from_fs(&mut self, path: &Path) {
        let folder_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        self.folder_name = folder_name.clone();

        let level_dat = path.join("level.dat");
        if level_dat.exists() {
            if let Ok(data) = std::fs::read(&level_dat) {
                self.load_from_level_dat(&data);
            }
        }

        let icon_path = path.join("icon.png");
        if icon_path.exists() {
            self.icon_file = icon_path.to_string_lossy().to_string();
        }

        self.size = calculate_dir_size(path);
        self.is_valid = true;
    }

    fn read_from_zip(&mut self, path: &Path) {
        self.folder_name = path.file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        let file = std::fs::File::open(path)
            .ok()
            .and_then(|f| zip::ZipArchive::new(f).ok());

        if let Some(mut archive) = file {
            for i in 0..archive.len() {
                let mut entry = archive.by_index(i).ok();
                if let Some(ref mut entry) = entry {
                    let name = entry.name().to_string();
                    if name == "level.dat" || name == "level.dat_old" {
                        let mut data = Vec::new();
                        if std::io::Read::read_to_end(entry, &mut data).is_ok() {
                            self.load_from_level_dat(&data);
                        }
                    } else if name == "icon.png" {
                        self.icon_file = name;
                    }
                }
            }
        }

        self.size = path.metadata().map(|m| m.len() as i64).unwrap_or(0);
        self.is_valid = true;
    }

    fn load_from_level_dat(&mut self, data: &[u8]) {
        // level.dat is a GZip'd NBT file
        // For now, do a basic scan for known strings/values
        if data.len() < 4 { return; }

        // Try to decompress with gzip
        use std::io::Read;
        let mut decoder = flate2::read::GzDecoder::new(data);
        let mut decompressed = Vec::new();
        if decoder.read_to_end(&mut decompressed).is_err() {
            return;
        }

        let content = String::from_utf8_lossy(&decompressed);

        // Extract world name
        if let Some(start) = content.find("LevelName") {
            if let Some(end) = content[start..].find('\n') {
                let line = &content[start..start + end];
                if let Some(name_start) = line.find('\'') {
                    if let Some(name_end) = line[name_start + 1..].find('\'') {
                        self.actual_name = line[name_start + 1..name_start + 1 + name_end].to_string();
                    }
                }
            }
        }

        if self.actual_name.is_empty() {
            self.actual_name = content.lines()
                .find(|l| l.contains("LevelName"))
                .and_then(|l| {
                    let parts: Vec<&str> = l.split(' ').collect();
                    parts.last().map(|s| s.trim_matches('\'').to_string())
                })
                .unwrap_or_else(|| self.folder_name.clone());
        }

        // Extract GameType
        if let Some(start) = content.find("GameType") {
            let rest = &content[start..];
            if let Some(end) = rest.find('\n') {
                let line = &rest[..end];
                for word in line.split(|c: char| !c.is_ascii_digit() && c != '-') {
                    if let Ok(val) = word.parse::<i32>() {
                        self.game_type = GameType::from_int(val);
                        break;
                    }
                }
            }
        }

        // Extract RandomSeed
        if let Some(start) = content.find("RandomSeed") {
            let rest = &content[start..];
            if let Some(end) = rest.find('\n') {
                let line = &rest[..end];
                for word in line.split(|c: char| !c.is_ascii_digit() && c != '-') {
                    if !word.is_empty() {
                        if let Ok(val) = word.parse::<i64>() {
                            self.random_seed = val;
                            break;
                        }
                    }
                }
            }
        }

        // Extract LastPlayed time
        if let Some(start) = content.find("LastPlayed") {
            let rest = &content[start..];
            if let Some(end) = rest.find('\n') {
                let line = &rest[..end];
                for word in line.split(|c: char| !c.is_ascii_digit()) {
                    if !word.is_empty() {
                        if let Ok(val) = word.parse::<i64>() {
                            self.last_played = val;
                            break;
                        }
                    }
                }
            }
        }
    }
}

fn calculate_dir_size(path: &Path) -> i64 {
    let mut total = 0i64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                total += path.metadata().map(|m| m.len() as i64).unwrap_or(0);
            } else if path.is_dir() {
                total += calculate_dir_size(&path);
            }
        }
    }
    total
}

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

    pub fn worlds(&self) -> &[World] { &self.worlds }

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

    pub fn len(&self) -> usize { self.worlds.len() }
    pub fn is_empty(&self) -> bool { self.worlds.is_empty() }
}
