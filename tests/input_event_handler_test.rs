use std::io::{stdout, Write};
use std::time::{Duration, Instant};

use anyhow::Result;
use async_trait::async_trait;
use crossterm::event::{self, Event as CEvent, KeyEvent};
use crossterm::{
    cursor::position,
    event::{poll, read, DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use tokio::sync::mpsc;

use megane::{
    event::Event,
    handler::{input_event_handler::InputEventHandler, EventHandler},
};

mod common;

#[tokio::test]
async fn test_input_event_handler() {
    let (input_tx, mut input_rx) = tokio::sync::mpsc::channel::<Event<KeyEvent>>(1);
    let mut handler = InputEventHandler::new(Duration::from_millis(100), input_tx, true);
    let handle = tokio::spawn(async move {
        handler.run().await.unwrap();
    });
    let event = input_rx.recv().await.unwrap();
    assert_eq!(Event::Tick, event);
    let _ = handle.await.unwrap();
}
