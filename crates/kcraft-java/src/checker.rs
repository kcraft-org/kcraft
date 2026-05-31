use std::collections::HashMap;
use std::process::{Command, Stdio};

use crate::version::JavaVersion;

#[derive(Debug, Clone)]
pub struct JavaCheckResult {
    pub path: String,
    pub mojang_platform: String,
    pub real_platform: String,
    pub java_version: JavaVersion,
    pub java_vendor: String,
    pub out_log: String,
    pub error_log: String,
    pub is_64bit: bool,
    pub id: i32,
    pub validity: JavaValidity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JavaValidity {
    Errored,
    ReturnedInvalidData,
    Valid,
}

pub struct JavaChecker {
    pub path: String,
    pub args: String,
    pub id: i32,
    pub min_mem: i32,
    pub max_mem: i32,
    pub perm_gen: i32,
}

impl Default for JavaChecker {
    fn default() -> Self {
        JavaChecker {
            path: String::new(),
            args: String::new(),
            id: 0,
            min_mem: 0,
            max_mem: 0,
            perm_gen: 64,
        }
    }
}

impl JavaChecker {
    pub fn new(path: String) -> Self {
        JavaChecker {
            path,
            ..Default::default()
        }
    }

    pub fn perform_check(&self) -> JavaCheckResult {
        let mut result = JavaCheckResult {
            path: self.path.clone(),
            mojang_platform: String::new(),
            real_platform: String::new(),
            java_version: JavaVersion::new(""),
            java_vendor: String::new(),
            out_log: String::new(),
            error_log: String::new(),
            is_64bit: false,
            id: self.id,
            validity: JavaValidity::Errored,
        };

        let checker_jar = get_java_check_path();
        if checker_jar.is_empty() || !std::path::Path::new(&checker_jar).exists() {
            tracing::warn!("Java checker jar not found at {}", checker_jar);
            return result;
        }

        let mut cmd = Command::new(&self.path);
        cmd.stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .envs(clean_environment());

        if !self.args.is_empty() {
            cmd.args(shell_words::split(&self.args).unwrap_or_default());
        }

        if self.min_mem != 0 {
            cmd.arg(format!("-Xms{}m", self.min_mem));
        }
        if self.max_mem != 0 {
            cmd.arg(format!("-Xmx{}m", self.max_mem));
        }
        if self.perm_gen != 64 {
            cmd.arg(format!("-XX:PermSize={}m", self.perm_gen));
        }

        cmd.arg("-jar").arg(&checker_jar);

        tracing::debug!(
            "Running java checker: {} {}",
            self.path,
            cmd.get_args()
                .map(|a| a.to_string_lossy())
                .collect::<Vec<_>>()
                .join(" ")
        );

        let output = match cmd.output() {
            Ok(o) => o,
            Err(e) => {
                tracing::error!("Failed to start java checker: {}", e);
                return result;
            }
        };

        result.out_log = String::from_utf8_lossy(&output.stdout).to_string();
        result.error_log = String::from_utf8_lossy(&output.stderr).to_string();

        let status = output.status;

        if !status.success() {
            tracing::warn!(
                "Java checker failed with exit code {:?}",
                status.code()
            );
            return result;
        }

        let mut results: HashMap<String, String> = HashMap::new();
        for line in result.out_log.lines() {
            let line = line.trim();
            if line.contains("/bedrock/strata") {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                if !key.is_empty() && !value.is_empty() {
                    results.insert(key.to_string(), value.to_string());
                }
            }
        }

        let os_arch = match results.get("os.arch") {
            Some(v) => v.clone(),
            None => {
                result.validity = JavaValidity::ReturnedInvalidData;
                return result;
            }
        };
        let java_version_str = match results.get("java.version") {
            Some(v) => v.clone(),
            None => {
                result.validity = JavaValidity::ReturnedInvalidData;
                return result;
            }
        };
        let java_vendor = match results.get("java.vendor") {
            Some(v) => v.clone(),
            None => {
                result.validity = JavaValidity::ReturnedInvalidData;
                return result;
            }
        };

        let is_64 = is_64bit_arch(&os_arch);

        result.validity = JavaValidity::Valid;
        result.is_64bit = is_64;
        result.mojang_platform = if is_64 { "64".to_string() } else { "32".to_string() };
        result.real_platform = os_arch;
        result.java_version = JavaVersion::new(&java_version_str);
        result.java_vendor = java_vendor;

        tracing::debug!("Java checker succeeded for {}", self.path);
        result
    }
}

fn is_64bit_arch(arch: &str) -> bool {
    matches!(
        arch,
        "x86_64"
            | "amd64"
            | "aarch64"
            | "aarch64_be"
            | "arm64"
            | "ppc64le"
            | "ppc64"
            | "riscv64"
            | "riscv64be"
            | "sparc64"
            | "mips64el"
            | "mips64"
            | "ia64"
    )
}

pub fn get_java_check_path() -> String {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_default();
    let jar_path = exe_dir.join("JavaCheck.jar");
    jar_path.to_string_lossy().to_string()
}

fn clean_environment() -> HashMap<String, String> {
    let mut env = HashMap::new();
    for (key, value) in std::env::vars_os() {
        env.insert(key.to_string_lossy().to_string(), value.to_string_lossy().to_string());
    }
    env
}
