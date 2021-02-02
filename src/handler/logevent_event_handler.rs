use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;

use super::*;
use crate::{
    client::LogClient,
    event::{LogEventEvent, TailLogEventEvent},
    state::{logevents_state::LogEventsState, search_state::SearchMode},
};

pub struct LogEventEventHandler {
    client: LogClient,
    state: Arc<Mutex<LogEventsState>>,
    inst_rx: mpsc::Receiver<LogEventEvent>,
    tail_inst_tx: mpsc::Sender<TailLogEventEvent>,
}

impl LogEventEventHandler {
    pub fn new(
        client: LogClient,
        state: Arc<Mutex<LogEventsState>>,
        inst_rx: mpsc::Receiver<LogEventEvent>,
        tail_inst_tx: mpsc::Sender<TailLogEventEvent>,
    ) -> Self {
        LogEventEventHandler {
            client,
            state,
            inst_rx,
            tail_inst_tx,
        }
    }
}

#[async_trait]
impl EventHandler for LogEventEventHandler {
    async fn run(&mut self) -> Result<()> {
        loop {
            if let Some(event) = self.inst_rx.recv().await {
                match event {
                    LogEventEvent::FetchLogEvents(gname, token, conditions, need_reset) => {
                        if let Some(condition) = conditions {
                            if let SearchMode::Tail = condition.mode {
                                self.tail_inst_tx
                                    .send(TailLogEventEvent::Start(
                                        gname,
                                        token,
                                        Some(condition),
                                        need_reset,
                                    ))
                                    .await
                                    .unwrap();
                            } else {
                                // TODO: try_lock()?
                                // TODO: error handling
                                self.tail_inst_tx
                                    .send(TailLogEventEvent::Stop)
                                    .await
                                    .unwrap();
                                self.state.lock().unwrap().is_fetching = true;
                                if need_reset {
                                    self.state.lock().unwrap().reset();
                                }
                                let (mut fetched_log_events, next_token) =
                                    self.client.fetch_logs(&gname, &token, &condition).await?;
                                self.state
                                    .lock()
                                    .unwrap()
                                    .events
                                    .push_items(&mut fetched_log_events, false);
                                self.state.lock().unwrap().next_token = next_token;
                                self.state.lock().unwrap().is_fetching = false;
                            }
                        }
                    }
                    LogEventEvent::Abort => {
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}
