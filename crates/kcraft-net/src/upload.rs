use std::sync::{Arc, RwLock};
use std::time::Duration;

use reqwest::Client;
use url::Url;

use crate::sink::{ByteArraySink, Sink};
use crate::{NetError, Result, TaskState};
use kcraft_core::BUILD_CONFIG;

pub struct Upload {
    url: Url,
    sink: ByteArraySink,
    data: Vec<u8>,
    state: TaskState,
}

impl Upload {
    pub fn new(url: Url, data: Vec<u8>) -> Self {
        let output = Arc::new(RwLock::new(Vec::new()));
        let sink = ByteArraySink::new(output);
        Upload {
            url,
            sink,
            data,
            state: TaskState::Inactive,
        }
    }

    pub fn make_byte_array(url: Url, output: Arc<RwLock<Vec<u8>>>, post_data: Vec<u8>) -> Self {
        let sink = ByteArraySink::new(output);
        Upload {
            url,
            sink,
            data: post_data,
            state: TaskState::Inactive,
        }
    }

    pub fn state(&self) -> TaskState {
        self.state
    }

    pub async fn execute(&mut self) -> Result<()> {
        if self.state == TaskState::AbortedByUser {
            return Err(NetError::Cancelled);
        }

        self.state = TaskState::Running;
        self.sink.init()?;

        let client = Client::builder()
            .user_agent(BUILD_CONFIG.user_agent.clone())
            .timeout(Duration::from_secs(120))
            .build()
            .map_err(|e| NetError::Network(e.to_string()))?;

        let response = client
            .post(self.url.clone())
            .header("Content-Type", "application/json")
            .body(self.data.clone())
            .send()
            .await
            .map_err(NetError::from)?;

        let status = response.status();
        if !status.is_success() {
            self.state = TaskState::Failed;
            return Err(NetError::HttpError(
                status.as_u16(),
                status.to_string(),
            ));
        }

        let bytes = response.bytes().await.map_err(NetError::from)?;
        self.sink.write(&bytes)?;
        self.sink.finalize()?;

        self.state = TaskState::Succeeded;
        Ok(())
    }

    pub fn abort(&mut self) {
        self.state = TaskState::AbortedByUser;
        let _ = self.sink.abort();
    }
}
