use std::sync::{Arc, Mutex};

use megane::{
    client::LogClient,
    event::LogGroupEvent,
    handler::{loggroup_event_handler::LogGroupEventHandler, EventHandler},
    state::loggroups_state::LogGroupsState,
};

mod common;

#[tokio::test]
async fn test_run() {
    let state = Arc::new(Mutex::new(LogGroupsState::new()));
    let (mut inst_tx, inst_rx) = tokio::sync::mpsc::channel::<LogGroupEvent>(1);
    let mock_client = common::get_mock_client("loggroups_01.json");
    let mut handler =
        LogGroupEventHandler::new(LogClient::new(mock_client), Arc::clone(&state), inst_rx);
    let handle = tokio::spawn(async move {
        handler.run().await.unwrap();
    });
    assert!(inst_tx.send(LogGroupEvent::FetchLogGroups).await.is_ok());
    assert!(inst_tx.send(LogGroupEvent::Abort).await.is_ok());

    let _ = handle.await.unwrap();

    for i in 0..=2 {
        assert_eq!(
            Some(format!("log_group_{}", (i + 1).to_string())),
            state
                .lock()
                .unwrap()
                .log_groups
                .get_item(i)
                .unwrap()
                .log_group_name
        );
        assert_eq!(
            Some(format!("{}", (i + 1).to_string())),
            state.lock().unwrap().log_groups.get_item(i).unwrap().arn,
        );
    }
    assert!(true);
}
