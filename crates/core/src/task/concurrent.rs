use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::task::runner::TaskRunner;

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

    fn can_abort(&self) -> bool {
        true
    }
    fn is_multistep(&self) -> bool {
        self.tasks.len() > 1
    }
}
