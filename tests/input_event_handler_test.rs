use std::time::Duration;

use crossterm::event::KeyEvent;
use tokio::sync::mpsc;

use megane::{
    event::Event,
    handler::{input_event_handler::InputEventHandler, EventHandler},
};

mod common;

#[tokio::test]
async fn test_input_event_handler() {
    let (input_tx, mut input_rx) = mpsc::channel::<Event<KeyEvent>>(1);
    let mut handler = InputEventHandler::new(Duration::from_millis(100), input_tx, true);
    let handle = tokio::spawn(async move {
        handler.run().await.unwrap();
    });
    let event = input_rx.recv().await.unwrap();
    assert_eq!(Event::Tick, event);
    let _ = handle.await.unwrap();
}
