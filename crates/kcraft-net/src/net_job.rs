use std::sync::Arc;

use tokio::sync::Semaphore;

use crate::download::Download;
use crate::upload::Upload;
use crate::{NetMode, Result, TaskState};
use tracing::debug;

pub enum NetActionKind {
    Download(Download),
    Upload(Upload),
}

pub struct NetJob {
    name: String,
    actions: Vec<NetActionKind>,
    state: TaskState,
    max_concurrent: usize,
    mode: NetMode,
    #[allow(dead_code)]
    retry_count: u32,
    max_retries: u32,
}

impl NetJob {
    pub fn new(name: &str) -> Self {
        NetJob {
            name: name.to_string(),
            actions: Vec::new(),
            state: TaskState::Inactive,
            max_concurrent: 6,
            mode: NetMode::Online,
            retry_count: 0,
            max_retries: 3,
        }
    }

    pub fn set_mode(&mut self, mode: NetMode) {
        self.mode = mode;
    }

    pub fn set_max_retries(&mut self, retries: u32) {
        self.max_retries = retries;
    }

    pub fn add_download(&mut self, download: Download) {
        self.actions.push(NetActionKind::Download(download));
    }

    pub fn add_upload(&mut self, upload: Upload) {
        self.actions.push(NetActionKind::Upload(upload));
    }

    pub fn state(&self) -> TaskState {
        self.state
    }

    pub fn progress(&self) -> (usize, usize) {
        (0, self.actions.len())
    }

    pub fn failed_count(&self) -> usize {
        0
    }

    pub async fn execute(&mut self) -> Result<()> {
        if self.state == TaskState::AbortedByUser {
            return Err(crate::NetError::Cancelled);
        }

        self.state = TaskState::Running;
        let total = self.actions.len();
        
        let temp_actions = std::mem::take(&mut self.actions);
        let mut actions_opt: Vec<Option<NetActionKind>> = temp_actions.into_iter().map(Some).collect();
        let mut failed_indices = Vec::new();

        for attempt in 0..self.max_retries {
            let semaphore = Arc::new(Semaphore::new(self.max_concurrent));

            let to_execute: Vec<usize> = if attempt == 0 {
                (0..total).collect()
            } else {
                failed_indices.clone()
            };

            if to_execute.is_empty() {
                break;
            }

            let mode = self.mode;
            let mut join_handles = Vec::new();

            let mut completed_count = 0;
            let total_to_execute = to_execute.len();

            for i in to_execute {
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let mut action = actions_opt[i].take().unwrap();
                
                join_handles.push(tokio::spawn(async move {
                    let res = match &mut action {
                        NetActionKind::Download(dl) => dl.execute(mode).await,
                        NetActionKind::Upload(up) => up.execute().await,
                    };
                    drop(permit);
                    (i, action, res)
                }));
            }

            failed_indices.clear();

            for handle in join_handles {
                let (i, returned_action, res) = handle.await.unwrap();
                actions_opt[i] = Some(returned_action);
                
                if let Err(e) = res {
                    failed_indices.push(i);
                    debug!("Action {} failed (attempt {}): {}", i, attempt + 1, e);
                } else {
                    completed_count += 1;
                    let _ = crate::NET_EVENTS.send(crate::NetEvent {
                        job_name: self.name.clone(),
                        total_actions: total_to_execute,
                        completed_actions: completed_count,
                        failed_actions: failed_indices.len(),
                    });
                }
            }

            if failed_indices.is_empty() {
                self.state = TaskState::Succeeded;
                self.actions = actions_opt.into_iter().flatten().collect();
                return Ok(());
            }

            if attempt + 1 < self.max_retries {
                debug!(
                    "NetJob '{}' attempt {}/{}: {} failed, retrying...",
                    self.name,
                    attempt + 1,
                    self.max_retries,
                    failed_indices.len()
                );
            } else {
                self.state = TaskState::Failed;
                self.actions = actions_opt.into_iter().flatten().collect();
                return Err(crate::NetError::Network(format!(
                    "{} actions failed in '{}' after {} attempts",
                    failed_indices.len(),
                    self.name,
                    self.max_retries
                )));
            }
        }

        self.state = TaskState::Failed;
        self.actions = actions_opt.into_iter().flatten().collect();
        Err(crate::NetError::Network(format!(
            "NetJob '{}' failed",
            self.name
        )))
    }

    pub fn abort(&mut self) {
        self.state = TaskState::AbortedByUser;
    }
}
