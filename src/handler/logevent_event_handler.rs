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
        while let LogEventEvent::FetchLogEvents(gname) = self.inst_rx.recv().await.unwrap() {
            // TODO: try_lock()?
            // TODO: error handling
            self.state.lock().unwrap().is_fetching = true;
            let (mut fetched_log_events, _state) = self.client.fetch_logs(&gname).await?;
            self.state
                .lock()
                .unwrap()
                .events
                .push_items(&mut fetched_log_events, None);
            self.state.lock().unwrap().is_fetching = false;
        }
        Ok(())
    }
}
