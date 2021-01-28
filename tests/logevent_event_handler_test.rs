use std::sync::{Arc, Mutex};

use megane::{
    client::LogClient,
    event::LogEventEvent,
    handler::{logevent_event_handler::LogEventEventHandler, EventHandler},
    state::logevents_state::LogEventsState,
};

mod common;

#[tokio::test]
async fn test_run() {
    let state = Arc::new(Mutex::new(LogEventsState::new()));
    let (mut inst_tx, inst_rx) = tokio::sync::mpsc::channel::<LogEventEvent>(1);
    let mock_client = common::get_mock_client("logevents_01.json");
    let mut handler =
        LogEventEventHandler::new(LogClient::new(mock_client), Arc::clone(&state), inst_rx);
    let handle = tokio::spawn(async move {
        handler.run().await.unwrap();
    });
    assert!(inst_tx
        .send(LogEventEvent::FetchLogEvents(
            "log group name".to_string(),
            None,
            None,
            true
        ))
        .await
        .is_ok());
    assert!(inst_tx.send(LogEventEvent::Abort).await.is_ok());

    let _ = handle.await.unwrap();

    for i in 0..=4 {
        assert_eq!(
            Some(format!("log_event_{}", (i + 1).to_string())),
            state.lock().unwrap().events.items().get(i).unwrap().message
        );
        assert_eq!(
            Some((i + 1).to_string()),
            state
                .lock()
                .unwrap()
                .events
                .items()
                .get(i)
                .unwrap()
                .event_id
        );
    }
}
