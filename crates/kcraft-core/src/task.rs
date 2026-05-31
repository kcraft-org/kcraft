use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Inactive,
    Running,
    Succeeded,
    Failed,
    AbortedByUser,
}

pub trait TaskRunner: Send {
    fn execute(&mut self) -> Result<(), String>;
    fn abort(&mut self) -> bool { false }
    fn can_abort(&self) -> bool { false }
    fn status(&self) -> &str { "" }
    fn progress(&self) -> (i64, i64) { (0, 100) }
    fn is_multistep(&self) -> bool { false }
    fn step_progress(&self) -> (i64, i64) { (0, 100) }
    fn step_status(&self) -> String { String::new() }
    fn warnings(&self) -> Vec<String> { Vec::new() }
    fn fail_reason(&self) -> String { String::new() }
}

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

    pub fn state(&self) -> TaskState { self.state }
    pub fn is_running(&self) -> bool { self.state == TaskState::Running }
    pub fn is_finished(&self) -> bool {
        matches!(self.state, TaskState::Succeeded | TaskState::Failed | TaskState::AbortedByUser)
    }
    pub fn was_successful(&self) -> bool { self.state == TaskState::Succeeded }

    pub fn status(&self) -> String { self.status.lock().unwrap().clone() }
    pub fn progress(&self) -> (i64, i64) { *self.progress.lock().unwrap() }
    pub fn fail_reason(&self) -> String { self.fail_reason.lock().unwrap().clone() }
    pub fn warnings(&self) -> Vec<String> { self.warnings.lock().unwrap().clone() }
    pub fn can_abort(&self) -> bool { self.can_abort }

    pub fn set_abortable(&mut self, can_abort: bool) { self.can_abort = can_abort; }
    pub fn set_status(&self, status: &str) { *self.status.lock().unwrap() = status.to_string(); }
    pub fn set_progress(&self, current: i64, total: i64) { *self.progress.lock().unwrap() = (current, total); }
    pub fn add_warning(&self, warning: &str) { self.warnings.lock().unwrap().push(warning.to_string()); }
    pub fn set_fail_reason(&self, reason: &str) { *self.fail_reason.lock().unwrap() = reason.to_string(); }

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
    fn default() -> Self { Self::new() }
}

#[allow(dead_code)]
pub struct SequentialTask {
    name: String,
    tasks: Vec<Box<dyn TaskRunner>>,
    current_index: usize,
    aborted: bool,
    fail_reason: String,
    warnings: Vec<String>,
}

impl SequentialTask {
    pub fn new(name: &str) -> Self {
        SequentialTask {
            name: name.to_string(),
            tasks: Vec::new(),
            current_index: 0,
            aborted: false,
            fail_reason: String::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_task(&mut self, task: Box<dyn TaskRunner>) {
        self.tasks.push(task);
    }
}

impl TaskRunner for SequentialTask {
    fn execute(&mut self) -> Result<(), String> {
        for i in 0..self.tasks.len() {
            if self.aborted {
                return Err("Aborted".to_string());
            }
            self.current_index = i;
            self.tasks[i].execute()?;
        }
        Ok(())
    }

    fn abort(&mut self) -> bool {
        self.aborted = true;
        if self.current_index < self.tasks.len() {
            self.tasks[self.current_index].abort();
        }
        true
    }

    fn can_abort(&self) -> bool { true }
    fn is_multistep(&self) -> bool { self.tasks.len() > 1 }
}

#[allow(dead_code)]
pub struct ConcurrentTask {
    name: String,
    tasks: Vec<Box<dyn TaskRunner>>,
    max_concurrent: usize,
    aborted: bool,
    fail_reason: String,
    warnings: Vec<String>,
}

impl ConcurrentTask {
    pub fn new(name: &str, max_concurrent: usize) -> Self {
        ConcurrentTask {
            name: name.to_string(),
            tasks: Vec::new(),
            max_concurrent,
            aborted: false,
            fail_reason: String::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_task(&mut self, task: Box<dyn TaskRunner>) {
        self.tasks.push(task);
    }
}

impl TaskRunner for ConcurrentTask {
    fn execute(&mut self) -> Result<(), String> {
        let aborted = Arc::new(AtomicBool::new(false));

        for task in &mut self.tasks {
            if aborted.load(Ordering::SeqCst) {
                break;
            }

            // In this simplified version, run sequentially with concurrency limit
            // A real implementation would use threads/async
            if let Err(e) = task.execute() {
                aborted.store(true, Ordering::SeqCst);
                return Err(e);
            }
        }
        Ok(())
    }

    fn abort(&mut self) -> bool {
        self.aborted = true;
        true
    }

    fn can_abort(&self) -> bool { true }
    fn is_multistep(&self) -> bool { self.tasks.len() > 1 }
}

#[allow(dead_code)]
pub struct MultipleOptionsTask {
    name: String,
    tasks: Vec<Box<dyn TaskRunner>>,
    current_index: usize,
}

impl MultipleOptionsTask {
    pub fn new(name: &str) -> Self {
        MultipleOptionsTask {
            name: name.to_string(),
            tasks: Vec::new(),
            current_index: 0,
        }
    }

    pub fn add_task(&mut self, task: Box<dyn TaskRunner>) {
        self.tasks.push(task);
    }
}

impl TaskRunner for MultipleOptionsTask {
    fn execute(&mut self) -> Result<(), String> {
        for i in 0..self.tasks.len() {
            self.current_index = i;
            if self.tasks[i].execute().is_ok() {
                return Ok(());
            }
        }
        Err("All options failed".to_string())
    }

    fn abort(&mut self) -> bool {
        if self.current_index < self.tasks.len() {
            self.tasks[self.current_index].abort();
        }
        true
    }

    fn can_abort(&self) -> bool { true }
    fn is_multistep(&self) -> bool { self.tasks.len() > 1 }
}
