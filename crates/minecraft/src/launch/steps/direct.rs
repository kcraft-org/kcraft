use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use crate::instance::MinecraftServerTarget;
use crate::launch::state::LaunchState;
use crate::launch::state::LogLevel;
use crate::launch::task::LaunchStep;
use crate::launch::task::LaunchTask;

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
