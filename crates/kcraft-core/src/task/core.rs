use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;

use crate::task::runner::TaskRunner;
use crate::task::state::TaskState;

pub struct Task {
    state: TaskState,
    abort_flag: Arc<AtomicBool>,
    status: Mutex<String>,
    progress: Mutex<(i64, i64)>,
    fail_reason: Mutex<String>,
    warnings: Mutex<Vec<String>>,
    can_abort: bool,
}

impl Task {
    pub fn new() -> Self {
        Task {
            state: TaskState::Inactive,
            abort_flag: Arc::new(AtomicBool::new(false)),
            status: Mutex::new(String::new()),
            progress: Mutex::new((0, 100)),
            fail_reason: Mutex::new(String::new()),
            warnings: Mutex::new(Vec::new()),
            can_abort: false,
        }
    }

    pub fn state(&self) -> TaskState {
        self.state
    }
    pub fn is_running(&self) -> bool {
        self.state == TaskState::Running
    }
    pub fn is_finished(&self) -> bool {
        matches!(
            self.state,
            TaskState::Succeeded | TaskState::Failed | TaskState::AbortedByUser
        )
    }
    pub fn was_successful(&self) -> bool {
        self.state == TaskState::Succeeded
    }

    pub fn status(&self) -> String {
        self.status.lock().unwrap().clone()
    }
    pub fn progress(&self) -> (i64, i64) {
        *self.progress.lock().unwrap()
    }
    pub fn fail_reason(&self) -> String {
        self.fail_reason.lock().unwrap().clone()
    }
    pub fn warnings(&self) -> Vec<String> {
        self.warnings.lock().unwrap().clone()
    }
    pub fn can_abort(&self) -> bool {
        self.can_abort
    }

    pub fn set_abortable(&mut self, can_abort: bool) {
        self.can_abort = can_abort;
    }
    pub fn set_status(&self, status: &str) {
        *self.status.lock().unwrap() = status.to_string();
    }
    pub fn set_progress(&self, current: i64, total: i64) {
        *self.progress.lock().unwrap() = (current, total);
    }
    pub fn add_warning(&self, warning: &str) {
        self.warnings.lock().unwrap().push(warning.to_string());
    }
    pub fn set_fail_reason(&self, reason: &str) {
        *self.fail_reason.lock().unwrap() = reason.to_string();
    }

    pub fn abort(&self) -> bool {
        if self.can_abort {
            self.abort_flag.store(true, Ordering::SeqCst);
            true
        } else {
            false
        }
    }

    pub fn run(&mut self, runner: &mut dyn TaskRunner) -> TaskState {
        self.state = TaskState::Running;

        let result = runner.execute();

        if self.abort_flag.load(Ordering::SeqCst) {
            self.state = TaskState::AbortedByUser;
        } else {
            self.state = match result {
                Ok(()) => TaskState::Succeeded,
                Err(reason) => {
                    self.set_fail_reason(&reason);
                    TaskState::Failed
                }
            };
        }

        // Copy warnings from runner
        for w in runner.warnings() {
            self.add_warning(&w);
        }

        self.state
    }
}

impl Default for Task {
    fn default() -> Self {
        Self::new()
    }
}
