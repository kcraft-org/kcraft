use std::path::PathBuf;

use crate::sink::base::Sink;
use crate::sink::file::FileSink;
use crate::{Result, TaskState};
use http_cache::MetaEntryPtr;
use md5::Digest;
use tracing::debug;

#[derive(Default)]
pub(crate) struct CacheHeaders {
    pub(crate) etag: Option<String>,
    pub(crate) last_modified: Option<String>,
    pub(crate) max_age: Option<i64>,
    pub(crate) age: Option<i64>,
    pub(crate) eternal: bool,
}

pub struct MetaCacheSink {
    file_sink: FileSink,
    entry: MetaEntryPtr,
    checksum: md5::Md5,
    digest: Option<String>,
    status_code: u16,
    cache_headers: CacheHeaders,
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
                debug!("Cache HIT: {} (etag={:?})", entry.relative_path, entry.etag);
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
        self.digest = Some(hex::encode(digest_bytes));

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

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
