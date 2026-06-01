use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use reqwest::Client;
use tracing::warn;
use url::Url;

use crate::sink::{ByteArraySink, FileSink, MetaCacheSink, Sink};
use crate::validator::Validator;
use crate::{NetError, NetMode, Result, TaskState};
use kcraft_core::BUILD_CONFIG;
use kcraft_http_cache::MetaEntryPtr;

pub struct Download {
    url: Url,
    sink: Box<dyn Sink>,
    options: DownloadOptions,
    state: TaskState,
}

#[derive(Default, Clone)]
pub struct DownloadOptions {
    pub accept_local: bool,
    pub eternal: bool,
    pub expected_etag: Option<String>,
}

impl Download {
    pub fn new(url: Url, sink: Box<dyn Sink>) -> Self {
        Download {
            url,
            sink,
            options: DownloadOptions::default(),
            state: TaskState::Inactive,
        }
    }

    pub fn make_cached(url: Url, entry: MetaEntryPtr) -> Self {
        let path = {
            let e = entry.read().unwrap();
            PathBuf::from(&e.base_path).join(&e.relative_path)
        };
        let sink = MetaCacheSink::new(path, entry);
        Download::new(url, Box::new(sink))
    }

    pub fn make_byte_array(url: Url, output: Arc<RwLock<Vec<u8>>>) -> Self {
        let sink = ByteArraySink::new(output);
        Download::new(url, Box::new(sink))
    }

    pub fn make_file(url: Url, path: PathBuf) -> Self {
        let sink = FileSink::new(path);
        Download::new(url, Box::new(sink))
    }

    pub fn set_options(&mut self, options: DownloadOptions) {
        self.options = options;
    }

    pub fn set_accept_local(&mut self, accept: bool) {
        self.options.accept_local = accept;
    }

    pub fn set_eternal(&mut self, eternal: bool) {
        self.options.eternal = eternal;
    }

    pub fn state(&self) -> TaskState {
        self.state
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn add_validator(&mut self, validator: Box<dyn Validator>) {
        if let Some(file_sink) = self.sink.as_any_mut().downcast_mut::<FileSink>() {
            file_sink.add_validator(validator);
        } else if let Some(ba_sink) = self.sink.as_any_mut().downcast_mut::<ByteArraySink>() {
            ba_sink.add_validator(validator);
        }
    }

    pub async fn execute(&mut self, mode: NetMode) -> Result<()> {
        if self.state == TaskState::AbortedByUser {
            return Err(NetError::Cancelled);
        }

        if mode == NetMode::Offline && self.sink.has_local_data() {
            self.state = TaskState::Succeeded;
            return Ok(());
        }

        self.state = TaskState::Running;
        let init_state = self.sink.init()?;

        if init_state == TaskState::Succeeded {
            self.state = TaskState::Succeeded;
            return Ok(());
        }

        let client = Client::builder()
            .user_agent(BUILD_CONFIG.user_agent.clone())
            .timeout(Duration::from_secs(60))
            .danger_accept_invalid_certs(false)
            .build()
            .map_err(|e| NetError::Network(e.to_string()))?;

        let mut req = client.get(self.url.clone());

        if let Some(ref etag) = self.options.expected_etag {
            req = req.header("If-None-Match", etag.clone());
        }

        let response = req.send().await.map_err(|e| {
            if self.options.accept_local && self.sink.has_local_data() {
                warn!("Download failed, using local data: {}", e);
                self.state = TaskState::Succeeded;
                return NetError::Network(e.to_string());
            }
            NetError::from(e)
        })?;

        if self.options.accept_local && self.sink.has_local_data() {
            self.state = TaskState::Succeeded;
            return Ok(());
        }

        if let Some(sink) = self.sink.as_any_mut().downcast_mut::<MetaCacheSink>() {
            sink.set_status_code(response.status().as_u16());
            sink.set_etag(
                response
                    .headers()
                    .get("etag")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string()),
            );
            sink.set_last_modified(
                response
                    .headers()
                    .get("last-modified")
                    .and_then(|v| v.to_str().ok())
                    .map(|s| s.to_string()),
            );
            if let Some(cache_control) = response.headers().get("cache-control") {
                if let Ok(val) = cache_control.to_str() {
                    if val.contains("max-age=") {
                        if let Some(pos) = val.find("max-age=") {
                            let age_str = &val[pos + 8..];
                            let age_end = age_str
                                .find(|c: char| !c.is_ascii_digit())
                                .unwrap_or(age_str.len());
                            if let Ok(max_age) = age_str[..age_end].parse::<i64>() {
                                sink.set_cache_control(Some(max_age));
                            }
                        }
                    }
                }
            }
        }

        if response.status().as_u16() == 304 {
            if let Some(sink) = self.sink.as_any_mut().downcast_mut::<MetaCacheSink>() {
                return sink.finalize().map(|_| ()).inspect_err(|_| {
                    self.state = TaskState::Failed;
                });
            }
        }

        let status = response.status();
        if !status.is_success() {
            self.state = TaskState::Failed;
            return Err(NetError::HttpError(status.as_u16(), status.to_string()));
        }

        let bytes = response.bytes().await.map_err(NetError::from)?;
        self.sink.write(&bytes)?;
        self.sink.finalize()?;

        if let Some(sink) = self.sink.as_any_mut().downcast_mut::<MetaCacheSink>() {
            sink.set_eternal(self.options.eternal);
        }

        self.state = TaskState::Succeeded;
        Ok(())
    }

    pub fn abort(&mut self) {
        self.state = TaskState::AbortedByUser;
        let _ = self.sink.abort();
    }
}
