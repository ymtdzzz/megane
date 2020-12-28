use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

use megane::{
    app::App,
    event::Event,
    handler::{main_event_handler::MainEventHandler, EventHandler},
    terminal::setup_terminal,
};

#[tokio::test]
async fn test_run() {
    let terminal = setup_terminal().unwrap();
    let (mut input_tx, input_rx) = mpsc::channel(1);
    let mut main_event_handler = MainEventHandler::new(terminal, App::default(), input_rx);
    let handle = tokio::spawn(async move {
        main_event_handler.run().await.unwrap();
    });
    assert!(input_tx.send(Event::Tick).await.is_ok());
    assert!(input_tx
        .send(Event::Input(KeyEvent::new(
            KeyCode::Char('a'),
            KeyModifiers::NONE
        )))
        .await
        .is_ok());
    assert!(input_tx
        .send(Event::Input(KeyEvent::new(
            KeyCode::Char('q'),
            KeyModifiers::NONE
        )))
        .await
        .is_ok());
    let _ = handle.await.unwrap();
}
