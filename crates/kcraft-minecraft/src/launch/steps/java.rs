use std::process::{Command, Stdio};

use crate::launch::state::LogLevel;
use crate::launch::task::LaunchStep;
use crate::launch::task::LaunchTask;

pub struct CheckJavaStep {
    java_path: String,
}

impl CheckJavaStep {
    pub fn new(java_path: &str) -> Self {
        CheckJavaStep {
            java_path: java_path.to_string(),
        }
    }
}

impl LaunchStep for CheckJavaStep {
    fn execute(&mut self, task: &mut LaunchTask) -> Result<(), String> {
        task.log(
            &format!("Checking Java version at: {}", self.java_path),
            LogLevel::Launcher,
        );

        let output = Command::new(&self.java_path)
            .arg("-version")
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to execute Java: {}", e))?;

        let version_str = String::from_utf8_lossy(&output.stderr);
        task.log(
            &format!("Java version: {}", version_str.trim()),
            LogLevel::Launcher,
        );

        let jv = crate::JavaVersion::new(version_str.trim());
        if !jv.is_parseable() {
            task.log("Warning: Could not parse Java version", LogLevel::Warning);
            return Ok(());
        }

        task.log(
            &format!("Java major version: {}", jv.major()),
            LogLevel::Launcher,
        );
        task.log(
            &format!("Java is 64-bit: {}", is_64bit_java(&self.java_path)),
            LogLevel::Launcher,
        );
        Ok(())
    }

    fn abort(&mut self) -> bool {
        true
    }
    fn can_abort(&self) -> bool {
        false
    }
    fn name(&self) -> &str {
        "CheckJava"
    }
}

fn is_64bit_java(java_path: &str) -> bool {
    let output = Command::new(java_path)
        .arg("-d64")
        .arg("-version")
        .stderr(Stdio::piped())
        .output();
    match output {
        Ok(o) => o.status.success(),
        Err(_) => false,
    }
}

pub struct VerifyJavaInstallStep;

impl LaunchStep for VerifyJavaInstallStep {
    fn execute(&mut self, task: &mut LaunchTask) -> Result<(), String> {
        let java_path = &task.instance.java_path;
        if java_path.is_empty() || !std::path::Path::new(java_path).exists() {
            return Err(format!("Java executable not found: {}", java_path));
        }

        let output = Command::new(java_path)
            .arg("-version")
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("Failed to execute Java: {}", e))?;

        if !output.status.success() {
            return Err("Java executable failed to run".to_string());
        }

        task.log("Java installation verified.", LogLevel::Launcher);
        Ok(())
    }
    fn abort(&mut self) -> bool {
        true
    }
    fn can_abort(&self) -> bool {
        false
    }
    fn name(&self) -> &str {
        "VerifyJavaInstall"
    }
}
