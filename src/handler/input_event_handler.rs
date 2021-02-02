use std::time::{Duration, Instant};

use anyhow::Result;
use async_trait::async_trait;
use crossterm::event::{self, Event as CEvent, KeyEvent};
use tokio::sync::mpsc;

use super::*;
use crate::{
    constant::TAIL_RATE,
    event::{Event, TailLogEventEvent},
};

pub struct InputEventHandler {
    tick_rate: Duration,
    input_tx: mpsc::Sender<Event<KeyEvent>>,
    tail_inst_txs: [mpsc::Sender<TailLogEventEvent>; 4],
    /// if true, run() will return Ok in short time.
    is_debug: bool,
}

impl InputEventHandler {
    pub fn new(
        tick_rate: Duration,
        input_tx: mpsc::Sender<Event<KeyEvent>>,
        tail_inst_txs: [mpsc::Sender<TailLogEventEvent>; 4],
        is_debug: bool,
    ) -> Self {
        InputEventHandler {
            tick_rate,
            input_tx,
            tail_inst_txs,
            is_debug,
        }
    }
}

#[async_trait]
impl EventHandler for InputEventHandler {
    async fn run(&mut self) -> Result<()> {
        let tail_tick_rate = *TAIL_RATE;
        let mut last_tail_tick = Instant::now();
        let mut last_tick = Instant::now();
        let start_time = Instant::now();
        let end_for = Duration::from_millis(120);
        loop {
            // KeyEvent handling
            if event::poll(self.tick_rate - last_tick.elapsed())? {
                if let CEvent::Key(key) = event::read()? {
                    self.input_tx.send(Event::Input(key)).await?;
                }
            }
            // Tick handling
            if last_tick.elapsed() >= self.tick_rate {
                self.input_tx.send(Event::Tick).await?;
                last_tick = Instant::now();
            }
            if last_tail_tick.elapsed() >= tail_tick_rate {
                for tx in self.tail_inst_txs.as_mut().iter_mut() {
                    tx.send(TailLogEventEvent::Tick).await.unwrap();
                }
                last_tail_tick = Instant::now();
            }
            if self.is_debug && start_time.elapsed() > end_for {
                break;
            }
        }
        Ok(())
    }
}
