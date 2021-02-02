use std::sync::{Arc, Mutex};

use megane::{
    client::LogClient,
    event::TailLogEventEvent,
    handler::{tail_logevent_event_handler::TailLogEventEventHandler, EventHandler},
    state::{
        logevents_state::LogEventsState,
        search_state::{SearchMode, SearchState},
    },
};

mod common;

#[tokio::test]
async fn test_run_basis() {
    // start tail mode and fetch some logs
    let state = Arc::new(Mutex::new(LogEventsState::new()));
    let (mut tail_inst_tx, tail_inst_rx) = tokio::sync::mpsc::channel::<TailLogEventEvent>(1);
    let mock_client = common::get_mock_client("logevents_01.json");
    let mut handler = TailLogEventEventHandler::new(
        LogClient::new(mock_client),
        Arc::clone(&state),
        tail_inst_rx,
    );
    let handle = tokio::spawn(async move {
        handler.run().await.unwrap();
    });
    let search_state = Some(SearchState::new(String::default(), SearchMode::TwelveHours));
    assert!(tail_inst_tx
        .send(TailLogEventEvent::Start(
            "log group name".to_string(),
            None,
            search_state,
            true
        ))
        .await
        .is_ok());
    assert!(tail_inst_tx.send(TailLogEventEvent::Tick).await.is_ok());
    assert!(tail_inst_tx.send(TailLogEventEvent::Abort).await.is_ok());

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

#[tokio::test]
async fn test_run_stop() {
    // fetch some logs but stop tail mode and delete all fetched logs
    let state = Arc::new(Mutex::new(LogEventsState::new()));
    let (mut tail_inst_tx, tail_inst_rx) = tokio::sync::mpsc::channel::<TailLogEventEvent>(1);
    let mock_client = common::get_mock_client("logevents_01.json");
    let mut handler = TailLogEventEventHandler::new(
        LogClient::new(mock_client),
        Arc::clone(&state),
        tail_inst_rx,
    );
    let handle = tokio::spawn(async move {
        handler.run().await.unwrap();
    });
    let search_state = Some(SearchState::new(String::default(), SearchMode::TwelveHours));
    assert!(tail_inst_tx
        .send(TailLogEventEvent::Start(
            "log group name".to_string(),
            None,
            search_state,
            true
        ))
        .await
        .is_ok());
    assert!(tail_inst_tx.send(TailLogEventEvent::Tick).await.is_ok());
    assert!(tail_inst_tx.send(TailLogEventEvent::Stop).await.is_ok());
    assert!(tail_inst_tx.send(TailLogEventEvent::Abort).await.is_ok());

    let _ = handle.await.unwrap();

    assert!(state.lock().unwrap().events.items().is_empty());
}
