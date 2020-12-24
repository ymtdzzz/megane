use std::time::{Duration, Instant};

use anyhow::Result;
use async_trait::async_trait;
use crossterm::event::{self, Event as CEvent, KeyEvent};
use tokio::sync::mpsc;

use super::*;
use crate::event::Event;

pub struct InputEventHandler {
    tick_rate: Duration,
    input_tx: mpsc::Sender<Event<KeyEvent>>,
    /// if true, run() will return Ok in short time.
    is_debug: bool,
}

impl InputEventHandler {
    pub fn new(
        tick_rate: Duration,
        input_tx: mpsc::Sender<Event<KeyEvent>>,
        is_debug: bool,
    ) -> Self {
        InputEventHandler {
            tick_rate,
            input_tx,
            is_debug,
        }
    }
}

#[async_trait]
impl EventHandler for InputEventHandler {
    async fn run(&mut self) -> Result<()> {
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
                // TODO: send Tick(s) for tail mode log areas?
                last_tick = Instant::now();
            }
            if self.is_debug {
                if start_time.elapsed() > end_for {
                    break;
                }
            }
        }
        Ok(())
    }
}
