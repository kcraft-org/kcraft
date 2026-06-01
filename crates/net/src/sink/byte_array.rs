use std::sync::{Arc, RwLock};

use crate::sink::base::Sink;
use crate::validator::Validator;
use crate::{NetError, Result, TaskState};

pub struct ByteArraySink {
    output: Arc<RwLock<Vec<u8>>>,
    validators: Vec<Box<dyn Validator>>,
}

impl ByteArraySink {
    pub fn new(output: Arc<RwLock<Vec<u8>>>) -> Self {
        ByteArraySink {
            output,
            validators: Vec::new(),
        }
    }

    pub fn add_validator(&mut self, validator: Box<dyn Validator>) {
        self.validators.push(validator);
    }
}

impl Sink for ByteArraySink {
    fn init(&mut self) -> Result<TaskState> {
        self.output.write().unwrap().clear();
        for v in &mut self.validators {
            v.init();
        }
        Ok(TaskState::Running)
    }

    fn write(&mut self, data: &[u8]) -> Result<TaskState> {
        self.output.write().unwrap().extend_from_slice(data);
        for v in &mut self.validators {
            v.write(data);
        }
        Ok(TaskState::Running)
    }

    fn abort(&mut self) -> Result<TaskState> {
        self.output.write().unwrap().clear();
        for v in &mut self.validators {
            v.abort();
        }
        Ok(TaskState::AbortedByUser)
    }

    fn finalize(&mut self) -> Result<TaskState> {
        for v in &mut self.validators {
            if !v.validate() {
                return Err(NetError::Validation("Validator failed".to_string()));
            }
        }
        Ok(TaskState::Succeeded)
    }

    fn has_local_data(&self) -> bool {
        !self.output.read().unwrap().is_empty()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
