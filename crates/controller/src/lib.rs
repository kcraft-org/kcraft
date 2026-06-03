use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ControllerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("No controller is currently connected")]
    NotConnected,
    #[error("Unsupported operation: {0}")]
    Unsupported(String),
}

pub type Result<T> = std::result::Result<T, ControllerError>;

#[derive(Debug, Clone)]
pub struct ControllerState {
    pub connected: bool,
    pub device_name: String,
    pub vendor_id: u16,
    pub product_id: u16,
}

#[derive(Debug, Clone)]
pub struct ControllerInput {
    pub axis_values: HashMap<String, f32>,
    pub button_pressed: HashMap<String, bool>,
    pub timestamp: std::time::Instant,
}

pub static DEFAULT_AXIS_MAP: std::sync::LazyLock<HashMap<&str, &str>> =
    std::sync::LazyLock::new(|| {
        let mut m = HashMap::new();
        m.insert("left_x", "0");
        m.insert("left_y", "1");
        m.insert("right_x", "2");
        m.insert("right_y", "3");
        m.insert("left_trigger", "4");
        m.insert("right_trigger", "5");
        m
    });

pub static DEFAULT_BUTTON_MAP: std::sync::LazyLock<HashMap<&str, u16>> =
    std::sync::LazyLock::new(|| {
        let mut m = HashMap::new();
        m.insert("a", 0);
        m.insert("b", 1);
        m.insert("x", 2);
        m.insert("y", 3);
        m.insert("left_bumper", 4);
        m.insert("right_bumper", 5);
        m.insert("back", 6);
        m.insert("start", 7);
        m.insert("guide", 8);
        m.insert("left_stick", 9);
        m.insert("right_stick", 10);
        m
    });

pub struct ControllerManager {
    connected: Mutex<bool>,
    device_path: Mutex<Option<PathBuf>>,
    button_remap: Mutex<HashMap<String, u16>>,
}

impl ControllerManager {
    pub fn new() -> Self {
        Self {
            connected: Mutex::new(false),
            device_path: Mutex::new(None),
            button_remap: Mutex::new(HashMap::new()),
        }
    }

    pub fn scan_devices() -> Result<Vec<ControllerState>> {
        let mut devices = Vec::new();

        #[cfg(target_os = "linux")]
        {
            let input_dir = PathBuf::from("/dev/input");
            if input_dir.is_dir() {
                for entry in std::fs::read_dir(&input_dir)? {
                    let entry = entry?;
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();

                    if name_str.starts_with("event") {
                        let device_name = guess_game_controller_name(&entry.path())
                            .unwrap_or_else(|| format!("/dev/input/{}", name_str));
                        devices.push(ControllerState {
                            connected: false,
                            device_name,
                            vendor_id: 0,
                            product_id: 0,
                        });
                    }
                }
            }
            tracing::debug!("Scanned /dev/input/, found {} event devices", devices.len());
        }

        #[cfg(not(target_os = "linux"))]
        {
            tracing::info!(
                "Controller scanning not implemented on this platform; returning empty list"
            );
        }

        Ok(devices)
    }

    pub fn connect(&self, device: &ControllerState) -> Result<()> {
        let mut connected = self.connected.lock().map_err(|e| {
            ControllerError::Io(std::io::Error::other(format!("lock poisoned: {e}")))
        })?;
        *connected = true;

        {
            let mut path = self.device_path.lock().map_err(|e| {
                ControllerError::Io(std::io::Error::other(format!("lock poisoned: {e}")))
            })?;
            *path = Some(PathBuf::from(&device.device_name));
        }

        tracing::info!("Connected to controller: {}", device.device_name);
        Ok(())
    }

    pub fn disconnect(&self) -> Result<()> {
        let mut connected = self.connected.lock().map_err(|e| {
            ControllerError::Io(std::io::Error::other(format!("lock poisoned: {e}")))
        })?;

        if !*connected {
            return Err(ControllerError::NotConnected);
        }

        *connected = false;
        {
            let mut path = self.device_path.lock().map_err(|e| {
                ControllerError::Io(std::io::Error::other(format!("lock poisoned: {e}")))
            })?;
            *path = None;
        }

        tracing::info!("Disconnected from controller");
        Ok(())
    }

    pub fn poll(&self) -> Result<Option<ControllerInput>> {
        let connected = self.connected.lock().map_err(|e| {
            ControllerError::Io(std::io::Error::other(format!("lock poisoned: {e}")))
        })?;

        if !*connected {
            return Err(ControllerError::NotConnected);
        }

        let input = ControllerInput {
            axis_values: HashMap::new(),
            button_pressed: HashMap::new(),
            timestamp: std::time::Instant::now(),
        };

        Ok(Some(input))
    }

    pub fn remap_button(&self, virtual_: &str, physical: u16) -> Result<()> {
        let mut remap = self.button_remap.lock().map_err(|e| {
            ControllerError::Io(std::io::Error::other(format!("lock poisoned: {e}")))
        })?;
        remap.insert(virtual_.to_string(), physical);
        tracing::debug!("Remapped virtual button '{virtual_}' to physical {physical}");
        Ok(())
    }

    pub fn get_button_remap(&self) -> Result<HashMap<String, u16>> {
        let remap = self.button_remap.lock().map_err(|e| {
            ControllerError::Io(std::io::Error::other(format!("lock poisoned: {e}")))
        })?;
        Ok(remap.clone())
    }

    pub fn is_connected(&self) -> Result<bool> {
        let connected = self.connected.lock().map_err(|e| {
            ControllerError::Io(std::io::Error::other(format!("lock poisoned: {e}")))
        })?;
        Ok(*connected)
    }
}

impl Default for ControllerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "linux")]
fn guess_game_controller_name(path: &std::path::Path) -> Option<String> {
    use std::fs;
    use std::io::Read;

    let device_name_path = path.with_file_name(
        path.file_name()?.to_string_lossy().replace("event", "js"),
    );

    if let Ok(mut f) = fs::File::open(path) {
        let mut buf = [0u8; 1024];
        if f.read(&mut buf).is_ok() {
            if let Ok(name) = String::from_utf8(buf.to_vec()) {
                let name = name.trim_matches('\0').to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }
    }

    if device_name_path.exists() {
        if let Ok(name) = std::fs::read_link(&device_name_path) {
            return Some(name.to_string_lossy().to_string());
        }
    }

    if let Ok(contents) = fs::read_to_string("/proc/bus/input/devices") {
        let event_name = path.file_name()?.to_string_lossy();
        for line in contents.lines() {
            if line.contains(&*event_name) {
                if let Some(next) = line.split('"').nth(1) {
                    return Some(next.to_string());
                }
            }
        }
    }

    None
}
