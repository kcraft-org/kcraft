use crate::task::runner::TaskRunner;

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

    fn can_abort(&self) -> bool {
        true
    }
    fn is_multistep(&self) -> bool {
        self.tasks.len() > 1
    }
}
