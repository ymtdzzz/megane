use std::{
    io::{stdout, Stdout, Write},
    time::Duration,
};

use anyhow::Result;
use clap::{crate_authors, crate_description, crate_name, crate_version, App as ClapApp};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rusoto_core::Region;
use rusoto_logs::CloudWatchLogsClient;
use scopeguard;
use tui::{backend::CrosstermBackend, Terminal};

use megane::{
    app::App,
    handler::{
        input_event_handler::InputEventHandler, main_event_handler::MainEventHandler, EventHandler,
    },
    terminal::*,
    ui::side_menu::SideMenu,
};

#[tokio::main]
async fn main() -> Result<()> {
    let _clap = ClapApp::new(crate_name!())
        .author(crate_authors!())
        .version(crate_version!())
        .about(crate_description!())
        .get_matches();

    //let _client = CloudWatchLogsClient::new(Region::ApNortheast1);

    // setup terminal
    let mut terminal = setup_terminal()?;

    // input handling
    let (input_tx, input_rx) = tokio::sync::mpsc::channel(1);
    tokio::spawn(async move {
        let mut input_event_handler =
            InputEventHandler::new(Duration::from_millis(1000), input_tx, false);
        // TODO: error handling
        let _ = input_event_handler.run().await;
    });

    let app: App<CrosstermBackend<Stdout>> = App::new(SideMenu::new(), vec![], false).await;

    terminal.clear()?;

    let mut main_event_handler = MainEventHandler::new(terminal, app, input_rx);

    main_event_handler.run().await?;

    Ok(())
}
