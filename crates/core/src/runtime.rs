use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeContext {
    pub java_architecture: String,
    pub java_real_architecture: String,
    pub java_path: String,
    pub system: String,
}

impl RuntimeContext {
    pub fn new() -> Self {
        RuntimeContext {
            java_architecture: String::new(),
            java_real_architecture: String::new(),
            java_path: String::new(),
            system: current_system().to_string(),
        }
    }

    pub fn classifier(&self) -> String {
        format!("{}-{}", self.system, self.java_architecture)
    }

    pub fn classifier_matches(&self, target: &str) -> bool {
        let classifier = self.classifier();
        classifier == target || target == self.system
    }

    pub fn current_system() -> &'static str {
        current_system()
    }
}

impl Default for RuntimeContext {
    fn default() -> Self {
        Self::new()
    }
}

fn current_system() -> &'static str {
    #[cfg(target_os = "linux")]
    {
        "linux"
    }
    #[cfg(target_os = "macos")]
    {
        "osx"
    }
    #[cfg(target_os = "windows")]
    {
        "windows"
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        "unknown"
    }
}
