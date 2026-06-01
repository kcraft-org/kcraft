use crate::{Result, TaskState};

pub trait Sink: Send + Sync {
    fn init(&mut self) -> Result<TaskState>;
    fn write(&mut self, data: &[u8]) -> Result<TaskState>;
    fn abort(&mut self) -> Result<TaskState>;
    fn finalize(&mut self) -> Result<TaskState>;
    fn has_local_data(&self) -> bool;
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
