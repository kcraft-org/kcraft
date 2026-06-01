use std::collections::HashMap;

use crate::instance::MinecraftInstance;
use crate::instance::MinecraftServerTarget;
use crate::launch::state::LaunchState;
use crate::launch::state::LogLevel;

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
