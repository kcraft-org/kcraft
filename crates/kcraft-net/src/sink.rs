use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use crate::validator::Validator;
use crate::{NetError, Result, TaskState};
use md5::Digest;
use kcraft_http_cache::MetaEntryPtr;
use tracing::debug;

pub trait Sink: Send + Sync {
    fn init(&mut self) -> Result<TaskState>;
    fn write(&mut self, data: &[u8]) -> Result<TaskState>;
    fn abort(&mut self) -> Result<TaskState>;
    fn finalize(&mut self) -> Result<TaskState>;
    fn has_local_data(&self) -> bool;
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

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
            std::fs::create_dir_all(parent)
                .map_err(NetError::Io)?;
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
        std::fs::rename(&self.tmp_path, &self.path)
            .map_err(NetError::Io)?;
        Ok(TaskState::Succeeded)
    }

    fn has_local_data(&self) -> bool {
        self.path.exists() && self.path.metadata().map(|m| m.len() > 0).unwrap_or(false)
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

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

    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}

pub struct MetaCacheSink {
    file_sink: FileSink,
    entry: MetaEntryPtr,
    checksum: md5::Md5,
    digest: Option<String>,
    status_code: u16,
    cache_headers: CacheHeaders,
}

#[derive(Default)]
struct CacheHeaders {
    etag: Option<String>,
    last_modified: Option<String>,
    max_age: Option<i64>,
    age: Option<i64>,
    eternal: bool,
}

impl MetaCacheSink {
    pub fn new(path: PathBuf, entry: MetaEntryPtr) -> Self {
        let file_sink = FileSink::new(path);
        MetaCacheSink {
            file_sink,
            entry,
            checksum: md5::Md5::new(),
            digest: None,
            status_code: 0,
            cache_headers: CacheHeaders::default(),
        }
    }

    pub fn set_status_code(&mut self, code: u16) {
        self.status_code = code;
    }

    pub fn set_etag(&mut self, etag: Option<String>) {
        self.cache_headers.etag = etag;
    }

    pub fn set_last_modified(&mut self, lm: Option<String>) {
        self.cache_headers.last_modified = lm;
    }

    pub fn set_cache_control(&mut self, max_age: Option<i64>) {
        self.cache_headers.max_age = max_age;
    }

    pub fn set_age(&mut self, age: Option<i64>) {
        self.cache_headers.age = age;
    }

    pub fn set_eternal(&mut self, eternal: bool) {
        self.cache_headers.eternal = eternal;
    }
}

impl Sink for MetaCacheSink {
    fn init(&mut self) -> Result<TaskState> {
        {
            let entry = self.entry.read().unwrap();
            if !entry.stale {
                debug!(
                    "Cache HIT: {} (etag={:?})",
                    entry.relative_path, entry.etag
                );
                return Ok(TaskState::Succeeded);
            }
        }
        self.file_sink.init()
    }

    fn write(&mut self, data: &[u8]) -> Result<TaskState> {
        self.checksum.update(data);
        self.file_sink.write(data)
    }

    fn abort(&mut self) -> Result<TaskState> {
        self.file_sink.abort()
    }

    fn finalize(&mut self) -> Result<TaskState> {
        let result = self.file_sink.finalize()?;
        if result != TaskState::Succeeded {
            return Ok(result);
        }

        if self.status_code == 304 {
            let mut entry = self.entry.write().unwrap();
            entry.stale = false;
            return Ok(TaskState::Succeeded);
        }

        let digest_bytes = self.checksum.finalize_reset();
        self.digest = Some(format!("{:x}", digest_bytes));

        let mut entry = self.entry.write().unwrap();

        if let Some(ref etag) = self.cache_headers.etag {
            entry.etag = etag.clone();
        }
        if let Some(ref lm) = self.cache_headers.last_modified {
            entry.remote_changed_timestamp = lm.clone();
        }
        if let Some(ref hash) = self.digest {
            entry.md5sum = hash.clone();
        }
        entry.local_changed_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .ok()
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        if self.cache_headers.eternal {
            entry.eternal = true;
            entry.max_age = i64::MAX;
            entry.current_age = 0;
        } else if let Some(max_age) = self.cache_headers.max_age {
            entry.max_age = max_age;
            entry.current_age = self.cache_headers.age.unwrap_or(0);
        } else {
            entry.max_age = 7 * 24 * 60 * 60;
            entry.current_age = 0;
        }
        entry.stale = false;

        Ok(TaskState::Succeeded)
    }

    fn has_local_data(&self) -> bool {
        self.file_sink.has_local_data()
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}
