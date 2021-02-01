use std::sync::{Arc, Mutex};

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;

use super::*;
use crate::{
    client::LogClient,
    event::TailLogEventEvent,
    state::{logevents_state::LogEventsState, search_state::SearchState},
};

pub struct TailLogEventEventHandler {
    client: LogClient,
    state: Arc<Mutex<LogEventsState>>,
    inst_rx: mpsc::Receiver<TailLogEventEvent>,
    tail_mode: bool,
    current_search_condition: SearchState,
}

impl TailLogEventEventHandler {
    pub fn new(
        client: LogClient,
        state: Arc<Mutex<LogEventsState>>,
        inst_rx: mpsc::Receiver<TailLogEventEvent>,
    ) -> Self {
        TailLogEventEventHandler {
            client,
            state,
            inst_rx,
            tail_mode: false,
            current_search_condition: SearchState::default(),
        }
    }
}

#[async_trait]
impl EventHandler for TailLogEventEventHandler {
    async fn run(&mut self) -> Result<()> {
        while let Some(event) = self.inst_rx.recv().await {
            match event {
                TailLogEventEvent::Start(gname, token, conditions, _need_reset) => {
                    if let Some(search_state) = conditions {
                        self.current_search_condition = search_state.clone();
                    }
                    self.state.lock().unwrap().next_token = token.clone();
                    self.state.lock().unwrap().current_log_group = Some(gname.clone());
                    self.state.lock().unwrap().reset();
                    self.tail_mode = true;
                }
                TailLogEventEvent::Stop => {
                    println!("tail stop");
                    self.tail_mode = false;
                    self.state.lock().unwrap().reset();
                }
                TailLogEventEvent::Tick => {
                    if self.tail_mode {
                        if !self.state.lock().unwrap().is_fetching {
                            // skip if fetching
                            self.state.lock().unwrap().is_fetching = true;
                            let gname = self.state.lock().unwrap().current_log_group.clone();
                            let token = self.state.lock().unwrap().next_token.clone();
                            let (mut fetched_log_events, next_token) = self
                                .client
                                .fetch_logs(&gname.unwrap(), &token, &self.current_search_condition)
                                .await?;
                            self.state
                                .lock()
                                .unwrap()
                                .events
                                .push_items(&mut fetched_log_events);
                            self.state.lock().unwrap().next_token = next_token;
                            self.state.lock().unwrap().cursor_last();
                            self.state.lock().unwrap().is_fetching = false;
                        }
                    }
                }
                TailLogEventEvent::Abort => {
                    break;
                }
            }
        }
        Ok(())
    }
}
