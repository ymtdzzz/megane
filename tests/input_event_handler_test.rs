use std::time::Duration;

use crossterm::event::KeyEvent;
use tokio::sync::mpsc;

use megane::{
    event::{Event, TailLogEventEvent},
    handler::{input_event_handler::InputEventHandler, EventHandler},
};

mod common;

#[tokio::test]
async fn test_input_event_handler() {
    let (input_tx, mut input_rx) = mpsc::channel::<Event<KeyEvent>>(1);
    let (tail_inst_tx_0, _tail_inst_rx_0) = mpsc::channel::<TailLogEventEvent>(1);
    let (tail_inst_tx_1, _tail_inst_rx_1) = mpsc::channel::<TailLogEventEvent>(1);
    let (tail_inst_tx_2, _tail_inst_rx_2) = mpsc::channel::<TailLogEventEvent>(1);
    let (tail_inst_tx_3, _tail_inst_rx_3) = mpsc::channel::<TailLogEventEvent>(1);
    let tail_inst_txs = [
        tail_inst_tx_0,
        tail_inst_tx_1,
        tail_inst_tx_2,
        tail_inst_tx_3,
    ];
    let mut handler =
        InputEventHandler::new(Duration::from_millis(100), input_tx, tail_inst_txs, true);
    let handle = tokio::spawn(async move {
        handler.run().await.unwrap();
    });
    let event = input_rx.recv().await.unwrap();
    assert_eq!(Event::Tick, event);
    let _ = handle.await.unwrap();
}
