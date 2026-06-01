use std::path::PathBuf;

use crate::sink::base::Sink;
use crate::validator::Validator;
use crate::{NetError, Result, TaskState};

pub struct FileSink {
    path: PathBuf,
    tmp_path: PathBuf,
    file: Option<std::fs::File>,
    validators: Vec<Box<dyn Validator>>,
}

impl FileSink {
    pub fn new(path: PathBuf) -> Self {
        let tmp_path = path.with_extension("tmp");
        FileSink {
            path,
            tmp_path,
            file: None,
            validators: Vec::new(),
        }
    }

    pub fn add_validator(&mut self, validator: Box<dyn Validator>) {
        self.validators.push(validator);
    }
}

impl Sink for FileSink {
    fn init(&mut self) -> Result<TaskState> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(NetError::Io)?;
        }
        let file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&self.tmp_path)
            .map_err(NetError::Io)?;
        self.file = Some(file);
        for v in &mut self.validators {
            v.init();
        }
        Ok(TaskState::Running)
    }

    fn write(&mut self, data: &[u8]) -> Result<TaskState> {
        if let Some(ref file) = self.file {
            use std::io::Write;
            let mut file_ref = file;
            file_ref.write_all(data).map_err(NetError::Io)?;
        }
        for v in &mut self.validators {
            v.write(data);
        }
        Ok(TaskState::Running)
    }

    fn abort(&mut self) -> Result<TaskState> {
        self.file = None;
        let _ = std::fs::remove_file(&self.tmp_path);
        for v in &mut self.validators {
            v.abort();
        }
        Ok(TaskState::AbortedByUser)
    }

    fn finalize(&mut self) -> Result<TaskState> {
        self.file = None;
        for v in &mut self.validators {
            if !v.validate() {
                let _ = std::fs::remove_file(&self.tmp_path);
                return Err(NetError::Validation("Validator failed".to_string()));
            }
        }
        std::fs::rename(&self.tmp_path, &self.path).map_err(NetError::Io)?;
        Ok(TaskState::Succeeded)
    }

    fn has_local_data(&self) -> bool {
        self.path.exists() && self.path.metadata().map(|m| m.len() > 0).unwrap_or(false)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
