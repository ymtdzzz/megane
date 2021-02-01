use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;

use super::*;
use crate::client::LogClient;
use crate::event::LogGroupEvent;
use crate::state::loggroups_state::LogGroupsState;

pub struct LogGroupEventHandler {
    client: LogClient,
    state: Arc<Mutex<LogGroupsState>>,
    inst_rx: mpsc::Receiver<LogGroupEvent>,
}

impl LogGroupEventHandler {
    pub fn new(
        client: LogClient,
        state: Arc<Mutex<LogGroupsState>>,
        inst_rx: mpsc::Receiver<LogGroupEvent>,
    ) -> Self {
        LogGroupEventHandler {
            client,
            state,
            inst_rx,
        }
    }
}

#[async_trait]
impl EventHandler for LogGroupEventHandler {
    async fn run(&mut self) -> Result<()> {
        loop {
            if let Some(event) = self.inst_rx.recv().await {
                match event {
                    LogGroupEvent::FetchLogGroups => {
                        // TODO: try_lock()?
                        // TODO: error handling
                        self.state.lock().unwrap().is_fetching = true;
                        let mut fetched_log_groups = self.client.fetch_log_groups().await?;
                        self.state
                            .lock()
                            .unwrap()
                            .log_groups
                            .push_items(&mut fetched_log_groups, false);
                        self.state.lock().unwrap().is_fetching = false;
                    }
                    _ => {
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}
