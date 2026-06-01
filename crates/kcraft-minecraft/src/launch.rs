use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use crate::assets::assets_utils;
use crate::instance::MinecraftInstance;
use crate::instance::MinecraftServerTarget;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchState {
    NotStarted,
    Running,
    Waiting,
    Failed,
    Aborted,
    Finished,
}

pub trait LaunchStep: Send {
    fn execute(&mut self, task: &mut LaunchTask) -> Result<(), String>;
    fn abort(&mut self) -> bool;
    fn can_abort(&self) -> bool {
        true
    }
    fn finalize(&mut self) {}
    fn proceed(&mut self) {}
    fn name(&self) -> &str;
}

pub struct LaunchTask {
    pub instance: MinecraftInstance,
    pub steps: Vec<Box<dyn LaunchStep>>,
    pub current_step: usize,
    pub state: LaunchState,
    pub log_lines: Vec<(String, LogLevel)>,
    pub pid: Option<u32>,
    pub censor_filter: HashMap<String, String>,
    pub session: Option<crate::AuthSession>,
    pub server: Option<MinecraftServerTarget>,
}

impl LaunchTask {
    pub fn new(instance: MinecraftInstance) -> Self {
        LaunchTask {
            instance,
            steps: Vec::new(),
            current_step: 0,
            state: LaunchState::NotStarted,
            log_lines: Vec::new(),
            pid: None,
            censor_filter: HashMap::new(),
            session: None,
            server: None,
        }
    }

    pub fn append_step(&mut self, step: Box<dyn LaunchStep>) {
        self.steps.push(step);
    }

    pub fn prepend_step(&mut self, step: Box<dyn LaunchStep>) {
        self.steps.insert(0, step);
    }

    pub fn execute(&mut self) -> Result<(), String> {
        self.state = LaunchState::Running;
        self.current_step = 0;

        while self.current_step < self.steps.len() {
            if self.state == LaunchState::Aborted {
                return Err("Launch aborted".to_string());
            }

            let mut step = self.steps.swap_remove(self.current_step);
            let result = step.execute(self);
            self.steps.insert(self.current_step, step);

            match result {
                Ok(()) => {
                    self.current_step += 1;
                }
                Err(e) => {
                    self.state = LaunchState::Failed;
                    // Finalize steps in reverse
                    for i in (0..=self.current_step).rev() {
                        self.steps[i].finalize();
                    }
                    return Err(e);
                }
            }
        }

        self.state = LaunchState::Finished;
        Ok(())
    }

    pub fn abort(&mut self) -> bool {
        match self.state {
            LaunchState::Aborted | LaunchState::Failed | LaunchState::Finished => return true,
            LaunchState::NotStarted => {
                self.state = LaunchState::Aborted;
                return true;
            }
            LaunchState::Running | LaunchState::Waiting => {
                if self.current_step < self.steps.len() {
                    let step = &mut self.steps[self.current_step];
                    if !step.can_abort() {
                        return false;
                    }
                    if step.abort() {
                        self.state = LaunchState::Aborted;
                        return true;
                    }
                }
            }
        }
        false
    }

    pub fn can_abort(&self) -> bool {
        match self.state {
            LaunchState::Aborted | LaunchState::Failed | LaunchState::Finished => false,
            LaunchState::NotStarted => true,
            LaunchState::Running | LaunchState::Waiting => {
                if self.current_step < self.steps.len() {
                    self.steps[self.current_step].can_abort()
                } else {
                    false
                }
            }
        }
    }

    pub fn proceed(&mut self) {
        if self.state == LaunchState::Waiting && self.current_step < self.steps.len() {
            self.steps[self.current_step].proceed();
        }
    }

    pub fn log(&mut self, line: &str, level: LogLevel) {
        let line = self.censor(line);
        self.log_lines.push((line.clone(), level));
    }

    pub fn censor(&self, text: &str) -> String {
        let mut result = text.to_string();
        for (key, value) in &self.censor_filter {
            result = result.replace(key, value);
        }
        result
    }

    pub fn set_censor_filter(&mut self, filter: HashMap<String, String>) {
        self.censor_filter = filter;
    }

    pub fn substitute_variables(&self, input: &str) -> String {
        let vars = self.instance.get_variables();
        let mut result = input.to_string();
        for (key, value) in &vars {
            result = result.replace(&format!("${}", key), value);
        }
        result
    }
}

#[derive(Debug, Clone)]
pub enum LogLevel {
    StdOut,
    StdErr,
    Warning,
    Error,
    Fatal,
    Launcher,
    Minecraft,
    Unknown,
}

// ============== STEP IMPLEMENTATIONS ==============

pub struct TextPrintStep {
    lines: Vec<String>,
    level: LogLevel,
}

impl TextPrintStep {
    pub fn new(line: &str, level: LogLevel) -> Self {
        TextPrintStep {
            lines: vec![line.to_string()],
            level,
        }
    }

    pub fn new_multi(lines: Vec<String>, level: LogLevel) -> Self {
        TextPrintStep { lines, level }
    }
}

impl LaunchStep for TextPrintStep {
    fn execute(&mut self, task: &mut LaunchTask) -> Result<(), String> {
        for line in &self.lines {
            task.log(line, self.level.clone());
        }
        Ok(())
    }

    fn abort(&mut self) -> bool {
        true
    }
    fn can_abort(&self) -> bool {
        true
    }
    fn name(&self) -> &str {
        "TextPrint"
    }
}

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

pub struct DirectJavaLaunchStep {
    working_dir: String,
    lib_dir: String,
    session: Option<crate::AuthSession>,
    server: Option<MinecraftServerTarget>,
}

impl DirectJavaLaunchStep {
    pub fn new(working_dir: &str) -> Self {
        DirectJavaLaunchStep {
            working_dir: working_dir.to_string(),
            lib_dir: String::new(),
            session: None,
            server: None,
        }
    }

    pub fn set_lib_dir(&mut self, lib_dir: &str) {
        self.lib_dir = lib_dir.to_string();
    }

    pub fn set_session(&mut self, session: crate::AuthSession) {
        self.session = Some(session);
    }

    pub fn set_server(&mut self, server: MinecraftServerTarget) {
        self.server = Some(server);
    }
}

impl LaunchStep for DirectJavaLaunchStep {
    fn execute(&mut self, task: &mut LaunchTask) -> Result<(), String> {
        let java_path = task.instance.java_path.clone();
        let native_path = task.instance.get_native_path();

        let raw_classpath = task.instance.get_class_path();
        let lib_base = if self.lib_dir.is_empty() {
            let inst_root = std::path::Path::new(&task.instance.instance_root);
            inst_root
                .parent()
                .and_then(|p| p.parent())
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default()
        } else {
            self.lib_dir.clone()
        };
        let classpath: Vec<String> = raw_classpath
            .iter()
            .map(|p| {
                let path = std::path::Path::new(&lib_base).join(p);
                if path.exists() {
                    path.to_string_lossy().to_string()
                } else {
                    p.clone()
                }
            })
            .collect();
        let classpath_str = classpath.join(":");

        let main_class = task.instance.get_main_class();
        let mut args = task.instance.java_arguments();

        args.push(format!("-Djava.library.path={}", native_path));
        args.push("-cp".to_string());
        args.push(classpath_str);
        args.push(main_class);

        let mc_args = task.instance.process_minecraft_args(
            self.session.as_ref().or(task.session.as_ref()),
            self.server.as_ref().or(task.server.as_ref()),
        );
        args.extend(mc_args);

        task.log(
            &format!("Java Arguments:\n[{}]", args.join(", ")),
            LogLevel::Launcher,
        );

        let mut cmd = Command::new(&java_path);
        cmd.args(&args)
            .current_dir(&self.working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Set environment variables
        let existing_ld = std::env::var("LD_LIBRARY_PATH").unwrap_or_default();
        if existing_ld.is_empty() {
            cmd.env("LD_LIBRARY_PATH", &native_path);
        } else {
            cmd.env(
                "LD_LIBRARY_PATH",
                format!("{}:{}", native_path, existing_ld),
            );
        }

        task.log(
            &format!("Starting Minecraft: {} {}", java_path, args.join(" ")),
            LogLevel::Launcher,
        );

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to launch Minecraft: {}", e))?;

        let pid = child.id();
        task.pid = Some(pid);
        task.log(
            &format!("Minecraft process ID: {}", pid),
            LogLevel::Launcher,
        );

        let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();

        let stdout_tx = tx.clone();
        let mut child_stdout = child.stdout.take();
        thread::spawn(move || {
            use std::io::Read;
            let mut buf = [0u8; 4096];
            if let Some(ref mut stdout) = child_stdout {
                while let Ok(n) = stdout.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    let _ = stdout_tx.send(String::from_utf8_lossy(&buf[..n]).to_string());
                }
            }
        });

        let stderr_tx = tx.clone();
        let mut child_stderr = child.stderr.take();
        thread::spawn(move || {
            use std::io::Read;
            let mut buf = [0u8; 4096];
            if let Some(ref mut stderr) = child_stderr {
                while let Ok(n) = stderr.read(&mut buf) {
                    if n == 0 {
                        break;
                    }
                    let _ = stderr_tx.send(String::from_utf8_lossy(&buf[..n]).to_string());
                }
            }
        });

        // Poll for output and process exit
        loop {
            match rx.try_recv() {
                Ok(line) => {
                    for l in line.lines() {
                        task.log(l, LogLevel::Minecraft);
                    }
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => break,
            }

            match child.try_wait() {
                Ok(Some(status)) => {
                    task.log(
                        &format!("Minecraft exited with code: {:?}", status.code()),
                        LogLevel::Launcher,
                    );
                    if let Some(code) = status.code() {
                        if code != 0 {
                            return Err("Game crashed.".to_string());
                        }
                    }
                    break;
                }
                Ok(None) => {
                    thread::sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    return Err(format!("Error waiting for Minecraft process: {}", e));
                }
            }

            if task.state == LaunchState::Aborted {
                let _ = child.kill();
                return Err("Launch aborted".to_string());
            }
        }

        Ok(())
    }

    fn abort(&mut self) -> bool {
        true
    }
    fn can_abort(&self) -> bool {
        true
    }
    fn finalize(&mut self) {}
    fn name(&self) -> &str {
        "DirectJavaLaunch"
    }
}

pub struct CreateGameFoldersStep;

impl LaunchStep for CreateGameFoldersStep {
    fn execute(&mut self, task: &mut LaunchTask) -> Result<(), String> {
        let instance = &task.instance;
        let dirs = [
            instance.game_root(),
            instance.bin_root(),
            instance.get_native_path(),
            instance.mods_root(),
            instance.core_mods_dir(),
            instance.resource_packs_dir(),
            instance.jar_mods_dir(),
        ];

        for dir in &dirs {
            std::fs::create_dir_all(dir)
                .map_err(|e| format!("Failed to create directory {}: {}", dir, e))?;
            task.log(&format!("Created directory: {}", dir), LogLevel::Launcher);
        }

        Ok(())
    }

    fn abort(&mut self) -> bool {
        true
    }
    fn can_abort(&self) -> bool {
        false
    }
    fn name(&self) -> &str {
        "CreateGameFolders"
    }
}

pub struct PreLaunchCommandStep;
pub struct PostLaunchCommandStep;
pub struct LookupServerAddressStep;
pub struct QuitAfterGameStopStep;
pub struct UpdateStep;
pub struct ClaimAccountStep;
pub struct ConfigureAuthlibInjectorStep;
pub struct ExtractNativesStep;
pub struct LauncherPartLaunchStep;
pub struct ModMinecraftJarStep;
pub struct PrintInstanceInfoStep;
pub struct ReconstructAssetsStep;
pub struct ScanModFoldersStep;
pub struct VerifyJavaInstallStep;

impl LaunchStep for PreLaunchCommandStep {
    fn execute(&mut self, task: &mut LaunchTask) -> Result<(), String> {
        task.log("Pre-launch command: not configured", LogLevel::Launcher);
        Ok(())
    }
    fn abort(&mut self) -> bool {
        true
    }
    fn can_abort(&self) -> bool {
        true
    }
    fn name(&self) -> &str {
        "PreLaunchCommand"
    }
}

impl LaunchStep for PostLaunchCommandStep {
    fn execute(&mut self, task: &mut LaunchTask) -> Result<(), String> {
        task.log("Post-launch command: not configured", LogLevel::Launcher);
        Ok(())
    }
    fn abort(&mut self) -> bool {
        true
    }
    fn can_abort(&self) -> bool {
        true
    }
    fn name(&self) -> &str {
        "PostLaunchCommand"
    }
}

impl LaunchStep for LookupServerAddressStep {
    fn execute(&mut self, task: &mut LaunchTask) -> Result<(), String> {
        task.log("Server address lookup: not applicable", LogLevel::Launcher);
        Ok(())
    }
    fn abort(&mut self) -> bool {
        true
    }
    fn can_abort(&self) -> bool {
        true
    }
    fn name(&self) -> &str {
        "LookupServerAddress"
    }
}

impl LaunchStep for QuitAfterGameStopStep {
    fn execute(&mut self, _task: &mut LaunchTask) -> Result<(), String> {
        Ok(())
    }
    fn abort(&mut self) -> bool {
        true
    }
    fn can_abort(&self) -> bool {
        false
    }
    fn name(&self) -> &str {
        "QuitAfterGameStop"
    }
}

impl LaunchStep for UpdateStep {
    fn execute(&mut self, task: &mut LaunchTask) -> Result<(), String> {
        task.log(
            "Update check: not implemented (offline mode)",
            LogLevel::Launcher,
        );
        Ok(())
    }
    fn abort(&mut self) -> bool {
        true
    }
    fn can_abort(&self) -> bool {
        true
    }
    fn name(&self) -> &str {
        "Update"
    }
}

impl LaunchStep for ClaimAccountStep {
    fn execute(&mut self, task: &mut LaunchTask) -> Result<(), String> {
        if let Some(ref session) = task.session {
            task.log(
                &format!("Using account: {}", session.player_name),
                LogLevel::Launcher,
            );
        }
        Ok(())
    }
    fn abort(&mut self) -> bool {
        true
    }
    fn can_abort(&self) -> bool {
        false
    }
    fn name(&self) -> &str {
        "ClaimAccount"
    }
}

impl LaunchStep for ConfigureAuthlibInjectorStep {
    fn execute(&mut self, task: &mut LaunchTask) -> Result<(), String> {
        task.log("Authlib-injector: not configured", LogLevel::Launcher);
        Ok(())
    }
    fn abort(&mut self) -> bool {
        true
    }
    fn can_abort(&self) -> bool {
        false
    }
    fn name(&self) -> &str {
        "ConfigureAuthlibInjector"
    }
}

impl LaunchStep for ExtractNativesStep {
    fn execute(&mut self, task: &mut LaunchTask) -> Result<(), String> {
        let native_path = task.instance.get_native_path();
        std::fs::create_dir_all(&native_path)
            .map_err(|e| format!("Failed to create natives directory: {}", e))?;

        // Clear previous natives
        if let Ok(entries) = std::fs::read_dir(&native_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let _ = std::fs::remove_file(&path);
                }
            }
        }

        // Extract native libraries from the native jars
        let native_jars = task.instance.get_native_jars();
        for jar_path in &native_jars {
            let path = std::path::Path::new(jar_path);
            if !path.exists() {
                task.log(
                    &format!("Native library not found: {}", jar_path),
                    LogLevel::Warning,
                );
                continue;
            }

            task.log(
                &format!("Extracting natives from: {}", jar_path),
                LogLevel::Launcher,
            );

            let file = std::fs::File::open(path)
                .map_err(|e| format!("Failed to open {}: {}", jar_path, e))?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|e| format!("Failed to read zip {}: {}", jar_path, e))?;

            for i in 0..archive.len() {
                let mut entry = archive
                    .by_index(i)
                    .map_err(|e| format!("Failed to read zip entry: {}", e))?;

                if entry.is_dir() {
                    continue;
                }

                let name = entry.name().to_string();
                // Only extract native library files (.so, .dll, .dylib)
                if !name.ends_with(".so")
                    && !name.ends_with(".dll")
                    && !name.ends_with(".dylib")
                    && !name.ends_with(".jnilib")
                {
                    continue;
                }

                let out_path = std::path::Path::new(&native_path).join(
                    std::path::Path::new(&name)
                        .file_name()
                        .unwrap_or(std::ffi::OsStr::new(&name)),
                );
                let mut outfile = std::fs::File::create(&out_path)
                    .map_err(|e| format!("Failed to create {}: {}", out_path.display(), e))?;
                std::io::copy(&mut entry, &mut outfile)
                    .map_err(|e| format!("Failed to extract {}: {}", name, e))?;
            }
        }

        task.log(
            &format!("Natives extracted to: {}", native_path),
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
    fn finalize(&mut self) {}
    fn name(&self) -> &str {
        "ExtractNatives"
    }
}

impl LaunchStep for ModMinecraftJarStep {
    fn execute(&mut self, _task: &mut LaunchTask) -> Result<(), String> {
        Ok(())
    }
    fn abort(&mut self) -> bool {
        true
    }
    fn can_abort(&self) -> bool {
        false
    }
    fn name(&self) -> &str {
        "ModMinecraftJar"
    }
}

impl LaunchStep for PrintInstanceInfoStep {
    fn execute(&mut self, task: &mut LaunchTask) -> Result<(), String> {
        let name = task.instance.name.clone();
        let root = task.instance.instance_root.clone();
        let game_root = task.instance.game_root();
        let java_path = task.instance.java_path.clone();
        let java_version = task.instance.java_version.clone();
        let min_mem = task.instance.min_mem;
        let max_mem = task.instance.max_mem;
        let main_class = task.instance.get_main_class();
        task.log(
            "======== Minecraft Instance Info ========",
            LogLevel::Launcher,
        );
        task.log(&format!("Instance: {}", name), LogLevel::Launcher);
        task.log(&format!("Root: {}", root), LogLevel::Launcher);
        task.log(&format!("Game dir: {}", game_root), LogLevel::Launcher);
        task.log(&format!("Java: {}", java_path), LogLevel::Launcher);
        task.log(
            &format!("Java version: {}", java_version),
            LogLevel::Launcher,
        );
        task.log(
            &format!("Memory: {} - {} MB", min_mem, max_mem),
            LogLevel::Launcher,
        );
        task.log(&format!("Main class: {}", main_class), LogLevel::Launcher);
        task.log(
            "========================================",
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
        "PrintInstanceInfo"
    }
}

impl LaunchStep for ReconstructAssetsStep {
    fn execute(&mut self, task: &mut LaunchTask) -> Result<(), String> {
        let assets = task
            .instance
            .components
            .get_profile()
            .map(|p| p.assets.clone())
            .unwrap_or_default();
        let resources_dir = task.instance.resources_dir();

        task.log("Reconstructing assets...", LogLevel::Launcher);
        assets_utils::reconstruct_assets(&assets, &resources_dir);
        task.log("Assets reconstructed.", LogLevel::Launcher);
        Ok(())
    }
    fn abort(&mut self) -> bool {
        true
    }
    fn can_abort(&self) -> bool {
        false
    }
    fn name(&self) -> &str {
        "ReconstructAssets"
    }
}

impl LaunchStep for ScanModFoldersStep {
    fn execute(&mut self, task: &mut LaunchTask) -> Result<(), String> {
        let mods_dir = task.instance.mods_root();
        if std::path::Path::new(&mods_dir).exists() {
            if let Ok(entries) = std::fs::read_dir(&mods_dir) {
                let count = entries.flatten().filter(|e| e.path().is_file()).count();
                task.log(
                    &format!("Mods folder contains {} files", count),
                    LogLevel::Launcher,
                );
            }
        }
        let coremods_dir = task.instance.core_mods_dir();
        if std::path::Path::new(&coremods_dir).exists() {
            if let Ok(entries) = std::fs::read_dir(&coremods_dir) {
                let count = entries.flatten().filter(|e| e.path().is_file()).count();
                task.log(
                    &format!("Coremods folder contains {} files", count),
                    LogLevel::Launcher,
                );
            }
        }
        Ok(())
    }
    fn abort(&mut self) -> bool {
        true
    }
    fn can_abort(&self) -> bool {
        false
    }
    fn name(&self) -> &str {
        "ScanModFolders"
    }
}

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

impl LaunchStep for LauncherPartLaunchStep {
    fn execute(&mut self, _task: &mut LaunchTask) -> Result<(), String> {
        // Launcher part launch is not used in standalone mode
        Ok(())
    }
    fn abort(&mut self) -> bool {
        true
    }
    fn can_abort(&self) -> bool {
        true
    }
    fn name(&self) -> &str {
        "LauncherPartLaunch"
    }
}
