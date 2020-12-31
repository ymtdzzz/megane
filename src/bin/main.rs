#![warn(rust_2018_idioms)]

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
    event::LogGroupEvent,
    handler::{
        input_event_handler::InputEventHandler, loggroup_event_handler::LogGroupEventHandler,
        main_event_handler::MainEventHandler, EventHandler,
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
    tokio::spawn(async {
        let mut loggroup_event_handler =
            LogGroupEventHandler::new(log_client, loggroup_state_clone, logg_inst_rx);
        let _ = loggroup_event_handler.run().await;
    });
    // fetch log groups at first
    let _ = logg_inst_tx.send(LogGroupEvent::FetchLogGroups).await;

    // setup app
    let app: App<CrosstermBackend<Stdout>> = App::new(
        SideMenu::new(Arc::clone(&loggroup_state)),
        vec![],
        logevent_states,
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
