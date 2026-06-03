use std::process::Command;

use crate::instance::Instance;
use crate::MinecraftError;

#[derive(Debug, Clone, Default)]
pub struct LaunchScriptConfig {
    pub pre_launch_command: Option<String>,
    pub post_launch_command: Option<String>,
    pub pre_launch_blocking: bool,
}

pub fn execute_pre_launch_script(
    config: &LaunchScriptConfig,
    instance: &Instance,
) -> std::result::Result<(), MinecraftError> {
    let cmd = match &config.pre_launch_command {
        Some(c) if !c.is_empty() => c.clone(),
        _ => return Ok(()),
    };

    let envs = build_env(instance);
    let shell = if cfg!(target_os = "windows") {
        "cmd"
    } else {
        "sh"
    };
    let shell_arg = if cfg!(target_os = "windows") {
        "/C"
    } else {
        "-c"
    };

    let mut child = Command::new(shell)
        .arg(shell_arg)
        .arg(&cmd)
        .envs(envs.iter().map(|(k, v)| (k.as_str(), v.as_str())))
        .spawn()
        .map_err(|e| {
            MinecraftError::ScriptExecution(format!("Failed to spawn pre-launch script: {}", e))
        })?;

    if config.pre_launch_blocking {
        let status = child.wait().map_err(|e| {
            MinecraftError::ScriptExecution(format!("Failed to wait for pre-launch script: {}", e))
        })?;
        if !status.success() {
            return Err(MinecraftError::ScriptExecution(format!(
                "Pre-launch script exited with code: {:?}",
                status.code()
            )));
        }
    }

    Ok(())
}

pub fn execute_post_launch_script(config: &LaunchScriptConfig, instance: &Instance) {
    let cmd = match &config.post_launch_command {
        Some(c) if !c.is_empty() => c.clone(),
        _ => return,
    };

    let envs = build_env(instance);
    let shell = if cfg!(target_os = "windows") {
        "cmd"
    } else {
        "sh"
    };
    let shell_arg = if cfg!(target_os = "windows") {
        "/C"
    } else {
        "-c"
    };

    std::thread::spawn(move || {
        let _ = Command::new(shell)
            .arg(shell_arg)
            .arg(&cmd)
            .envs(envs.iter().map(|(k, v)| (k.as_str(), v.as_str())))
            .spawn();
    });
}

fn build_env(instance: &Instance) -> Vec<(String, String)> {
    let mut envs = Vec::new();

    envs.push(("INSTANCE_ID".to_string(), instance.id()));
    envs.push(("INSTANCE_NAME".to_string(), instance.name.clone()));
    envs.push(("INSTANCE_DIR".to_string(), instance.instance_root.clone()));

    let mc_version = instance
        .components
        .get_component_version("net.minecraft")
        .unwrap_or("unknown")
        .to_string();
    envs.push(("MC_VERSION".to_string(), mc_version));

    let loaders = instance.components.get_mod_loaders();
    let loader_str = if loaders.is_empty() {
        "vanilla".to_string()
    } else {
        loaders.join("+")
    };
    envs.push(("LOADER".to_string(), loader_str));

    envs.push(("JAVA_PATH".to_string(), instance.java_path.clone()));
    envs.push(("MIN_MEM".to_string(), instance.min_mem.to_string()));
    envs.push(("MAX_MEM".to_string(), instance.max_mem.to_string()));

    envs
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_build_env() {
        let mut instance = Instance::new("/tmp/test_instance", "TestInstance");
        instance.java_path = "/usr/bin/java".to_string();
        instance.min_mem = 1024;
        instance.max_mem = 4096;

        let envs = build_env(&instance);
        assert!(envs.contains(&("INSTANCE_NAME".to_string(), "TestInstance".to_string())));
        assert!(envs.contains(&("JAVA_PATH".to_string(), "/usr/bin/java".to_string())));
        assert!(envs.contains(&("MIN_MEM".to_string(), "1024".to_string())));
        assert!(envs.contains(&("MAX_MEM".to_string(), "4096".to_string())));
    }

    #[test]
    fn test_empty_config() {
        let instance = Instance::new("/tmp/test", "test");
        let config = LaunchScriptConfig::default();
        assert!(execute_pre_launch_script(&config, &instance).is_ok());
    }
}
