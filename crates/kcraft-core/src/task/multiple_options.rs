use crate::task::runner::TaskRunner;

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

    fn can_abort(&self) -> bool {
        true
    }
    fn is_multistep(&self) -> bool {
        self.tasks.len() > 1
    }
}
