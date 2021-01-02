#![warn(rust_2018_idioms)]
#![feature(destructuring_assignment)]

use std::{
    io::Stdout,
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::Result;
use clap::{crate_authors, crate_description, crate_name, crate_version, App as ClapApp};
use rusoto_core::Region;
use rusoto_logs::CloudWatchLogsClient;
use tokio::sync::mpsc;
use tui::backend::CrosstermBackend;

use megane::{
    app::App,
    client::LogClient,
    constant::HELP_INSTRUCTION,
    event::{LogEventEvent, LogGroupEvent},
    handler::{
        input_event_handler::InputEventHandler, logevent_event_handler::LogEventEventHandler,
        loggroup_event_handler::LogGroupEventHandler, main_event_handler::MainEventHandler,
        EventHandler,
    },
    state::{
        logevents_state::LogEventsState, loggroups_state::LogGroupsState,
        status_bar_state::StatusBarState,
    },
    terminal::*,
    ui::{side_menu::SideMenu, status_bar::StatusBar},
};

#[tokio::main]
async fn main() -> Result<()> {
    let _clap = ClapApp::new(crate_name!())
        .author(crate_authors!())
        .version(crate_version!())
        .about(crate_description!())
        .get_matches();

    // setup terminal
    let mut terminal = setup_terminal()?;

    // setup states and client
    let aws_client = CloudWatchLogsClient::new(Region::ApNortheast1);
    let log_client = LogClient::new(aws_client);
    let loggroup_state = Arc::new(Mutex::new(LogGroupsState::new()));
    let status_bar_state = Arc::new(Mutex::new(StatusBarState::new(HELP_INSTRUCTION.clone())));
    let logevent_states = [
        Arc::new(Mutex::new(LogEventsState::default())),
        Arc::new(Mutex::new(LogEventsState::default())),
        Arc::new(Mutex::new(LogEventsState::default())),
        Arc::new(Mutex::new(LogEventsState::default())),
    ];
    let logevent_state_clone_0 = Arc::clone(&logevent_states[0]);
    let logevent_state_clone_1 = Arc::clone(&logevent_states[1]);
    let logevent_state_clone_2 = Arc::clone(&logevent_states[2]);
    let logevent_state_clone_3 = Arc::clone(&logevent_states[3]);

    // input event handling
    let (input_tx, input_rx) = tokio::sync::mpsc::channel(1);
    tokio::spawn(async move {
        let mut input_event_handler =
            InputEventHandler::new(Duration::from_millis(100), input_tx, false);
        let _ = input_event_handler.run().await;
    });

    // loggroup event handling
    let (mut logg_inst_tx, logg_inst_rx) = mpsc::channel(1);
    let loggroup_state_clone = Arc::clone(&loggroup_state);
    let log_client_clone = log_client.clone();
    tokio::spawn(async {
        let mut loggroup_event_handler =
            LogGroupEventHandler::new(log_client_clone, loggroup_state_clone, logg_inst_rx);
        let _ = loggroup_event_handler.run().await;
    });
    // fetch log groups at first
    let _ = logg_inst_tx.send(LogGroupEvent::FetchLogGroups).await;

    // logevent event handling
    let (logevent_inst_tx_0, logevent_inst_rx_0) = mpsc::channel(1);
    let (logevent_inst_tx_1, logevent_inst_rx_1) = mpsc::channel(1);
    let (logevent_inst_tx_2, logevent_inst_rx_2) = mpsc::channel(1);
    let (logevent_inst_tx_3, logevent_inst_rx_3) = mpsc::channel(1);
    let log_client_clone = log_client.clone();
    tokio::spawn(async move {
        let mut logevent_event_handler =
            LogEventEventHandler::new(log_client_clone, logevent_state_clone_0, logevent_inst_rx_0);
        let _ = logevent_event_handler.run().await;
    });
    let log_client_clone = log_client.clone();
    tokio::spawn(async move {
        let mut logevent_event_handler =
            LogEventEventHandler::new(log_client_clone, logevent_state_clone_1, logevent_inst_rx_1);
        let _ = logevent_event_handler.run().await;
    });
    let log_client_clone = log_client.clone();
    tokio::spawn(async move {
        let mut logevent_event_handler =
            LogEventEventHandler::new(log_client_clone, logevent_state_clone_2, logevent_inst_rx_2);
        let _ = logevent_event_handler.run().await;
    });
    let log_client_clone = log_client.clone();
    tokio::spawn(async move {
        let mut logevent_event_handler =
            LogEventEventHandler::new(log_client_clone, logevent_state_clone_3, logevent_inst_rx_3);
        let _ = logevent_event_handler.run().await;
    });

    // setup app
    let app: App<CrosstermBackend<Stdout>> = App::new(
        SideMenu::new(Arc::clone(&loggroup_state)),
        vec![],
        logevent_states,
        [
            logevent_inst_tx_0,
            logevent_inst_tx_1,
            logevent_inst_tx_2,
            logevent_inst_tx_3,
        ],
        StatusBar::new(status_bar_state),
        false,
        false,
    )
    .await;

    terminal.clear()?;

    let mut main_event_handler = MainEventHandler::new(terminal, app, input_rx);

    main_event_handler.run().await?;

    Ok(())
}
