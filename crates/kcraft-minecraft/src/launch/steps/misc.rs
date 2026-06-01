use crate::launch::state::LogLevel;
use crate::launch::task::LaunchStep;
use crate::launch::task::LaunchTask;

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

pub struct PreLaunchCommandStep;
pub struct PostLaunchCommandStep;
pub struct LookupServerAddressStep;
pub struct QuitAfterGameStopStep;
pub struct UpdateStep;
pub struct ClaimAccountStep;
pub struct ConfigureAuthlibInjectorStep;
pub struct LauncherPartLaunchStep;
pub struct ModMinecraftJarStep;
pub struct PrintInstanceInfoStep;

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

impl LaunchStep for LauncherPartLaunchStep {
    fn execute(&mut self, _task: &mut LaunchTask) -> Result<(), String> {
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
