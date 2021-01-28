use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;

use super::*;
use crate::{client::LogClient, event::LogEventEvent, state::logevents_state::LogEventsState};

pub struct LogEventEventHandler {
    client: LogClient,
    state: Arc<Mutex<LogEventsState>>,
    inst_rx: mpsc::Receiver<LogEventEvent>,
}

impl LogEventEventHandler {
    pub fn new(
        client: LogClient,
        state: Arc<Mutex<LogEventsState>>,
        inst_rx: mpsc::Receiver<LogEventEvent>,
    ) -> Self {
        LogEventEventHandler {
            client,
            state,
            inst_rx,
        }
    }
}

#[async_trait]
impl EventHandler for LogEventEventHandler {
    async fn run(&mut self) -> Result<()> {
        while let LogEventEvent::FetchLogEvents(gname, token, conditions, need_reset) =
            self.inst_rx.recv().await.unwrap()
        {
            // TODO: try_lock()?
            // TODO: error handling
            self.state.lock().unwrap().is_fetching = true;
            if need_reset {
                self.state.lock().unwrap().reset();
            }
            let (mut fetched_log_events, next_token) =
                self.client.fetch_logs(&gname, token, conditions).await?;
            self.state
                .lock()
                .unwrap()
                .events
                .push_items(&mut fetched_log_events, None);
            self.state.lock().unwrap().next_token = next_token;
            self.state.lock().unwrap().is_fetching = false;
        }
        Ok(())
    }
}
