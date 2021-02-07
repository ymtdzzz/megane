#![warn(rust_2018_idioms)]
#![feature(destructuring_assignment)]

use std::{
    io::Stdout,
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::Result;
use clap::{crate_authors, crate_description, crate_name, crate_version, App as ClapApp, Arg};
use log::*;
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use tokio::sync::mpsc;
use tui::backend::CrosstermBackend;

use megane::{
    app::App,
    client::LogClient,
    constant::HELP_INSTRUCTION,
    event::LogGroupEvent,
    handler::{
        input_event_handler::InputEventHandler, logevent_event_handler::LogEventEventHandler,
        loggroup_event_handler::LogGroupEventHandler, main_event_handler::MainEventHandler,
        tail_logevent_event_handler::TailLogEventEventHandler, EventHandler,
    },
    state::{
        logevents_state::LogEventsState, loggroups_state::LogGroupsState,
        status_bar_state::StatusBarState,
    },
    terminal::*,
    ui::{side_menu::SideMenu, status_bar::StatusBar},
    utils::get_aws_client,
};

#[tokio::main]
async fn main() -> Result<()> {
    let clap = ClapApp::new(crate_name!())
        .author(crate_authors!())
        .version(crate_version!())
        .about(crate_description!())
        .arg(
            Arg::with_name("profile")
                .required(false)
                .long("profile")
                .short("p")
                .takes_value(true)
                .help("Specific AWS profile. If not provided, default profile will be used."),
        )
        .arg(
            Arg::with_name("region")
                .required(false)
                .long("region")
                .short("r")
                .takes_value(true)
                .help("Specific AWS region. If not provided, default region will be used."),
        )
        .arg(
            Arg::with_name("role_arn")
                .required(false)
                .long("role_arn")
                .short("a")
                .takes_value(true)
                .help("The role arn you want to assume."),
        )
        .arg(
            Arg::with_name("role_name")
                .required(false)
                .long("role_name")
                .short("n")
                .takes_value(true)
                .help("The role name you want to assume. Ensure that your current credential is allowed to action 'iam:GetRole'"),
        )
        .arg(
            Arg::with_name("debug_mode")
                .required(false)
                .long("debug")
                .short("d")
                .help("Debug mode. Events will be written to ./log/output.log ."),
        )
        .get_matches();

    // setup logging
    if clap.is_present("debug_mode") {
        let logfile = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::new(
                "{l}[{d(%Y-%m-%d %H:%M:%S)}] - {m}\n",
            )))
            .build("log/output.log")?;
        let config = Config::builder()
            .appender(Appender::builder().build("logfile", Box::new(logfile)))
            .build(Root::builder().appender("logfile").build(LevelFilter::Info))?;
        log4rs::init_config(config)?;
        std::panic::set_hook(Box::new(|_panic_info| {
            log::error!("{:?}", backtrace::Backtrace::new());
        }));
    }

    // setup states and client
    //let aws_client = CloudWatchLogsClient::new(Region::ApNortheast1);
    let aws_client = get_aws_client(
        clap.value_of("profile"),
        clap.value_of("region"),
        clap.value_of("role_name"),
        clap.value_of("role_arn"),
    )
    .await?;
    let log_client = LogClient::new(aws_client);
    // setup terminal
    let mut terminal = setup_terminal()?;
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
    // for tail mode
    let logevent_state_clone_0_clone = Arc::clone(&logevent_states[0]);
    let logevent_state_clone_1_clone = Arc::clone(&logevent_states[1]);
    let logevent_state_clone_2_clone = Arc::clone(&logevent_states[2]);
    let logevent_state_clone_3_clone = Arc::clone(&logevent_states[3]);

    // input event handling
    let (tail_logevent_inst_tx_0, tail_logevent_inst_rx_0) = mpsc::channel(1);
    let (tail_logevent_inst_tx_1, tail_logevent_inst_rx_1) = mpsc::channel(1);
    let (tail_logevent_inst_tx_2, tail_logevent_inst_rx_2) = mpsc::channel(1);
    let (tail_logevent_inst_tx_3, tail_logevent_inst_rx_3) = mpsc::channel(1);
    let tail_logevent_inst_txs = [
        tail_logevent_inst_tx_0.clone(),
        tail_logevent_inst_tx_1.clone(),
        tail_logevent_inst_tx_2.clone(),
        tail_logevent_inst_tx_3.clone(),
    ];
    let (input_tx, input_rx) = tokio::sync::mpsc::channel(1);
    tokio::spawn(async move {
        let mut input_event_handler = InputEventHandler::new(
            Duration::from_millis(100),
            input_tx,
            tail_logevent_inst_txs,
            false,
        );
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
        let mut logevent_event_handler = LogEventEventHandler::new(
            log_client_clone,
            logevent_state_clone_0,
            logevent_inst_rx_0,
            tail_logevent_inst_tx_0,
        );
        let _ = logevent_event_handler.run().await;
    });
    let log_client_clone = log_client.clone();
    tokio::spawn(async move {
        let mut logevent_event_handler = LogEventEventHandler::new(
            log_client_clone,
            logevent_state_clone_1,
            logevent_inst_rx_1,
            tail_logevent_inst_tx_1,
        );
        let _ = logevent_event_handler.run().await;
    });
    let log_client_clone = log_client.clone();
    tokio::spawn(async move {
        let mut logevent_event_handler = LogEventEventHandler::new(
            log_client_clone,
            logevent_state_clone_2,
            logevent_inst_rx_2,
            tail_logevent_inst_tx_2,
        );
        let _ = logevent_event_handler.run().await;
    });
    let log_client_clone = log_client.clone();
    tokio::spawn(async move {
        let mut logevent_event_handler = LogEventEventHandler::new(
            log_client_clone,
            logevent_state_clone_3,
            logevent_inst_rx_3,
            tail_logevent_inst_tx_3,
        );
        let _ = logevent_event_handler.run().await;
    });

    // tail logevent event handling
    let log_client_clone = log_client.clone();
    tokio::spawn(async move {
        let mut tail_logevent_event_handler = TailLogEventEventHandler::new(
            log_client_clone,
            logevent_state_clone_0_clone,
            tail_logevent_inst_rx_0,
        );
        let _ = tail_logevent_event_handler.run().await;
    });
    let log_client_clone = log_client.clone();
    tokio::spawn(async move {
        let mut tail_logevent_event_handler = TailLogEventEventHandler::new(
            log_client_clone,
            logevent_state_clone_1_clone,
            tail_logevent_inst_rx_1,
        );
        let _ = tail_logevent_event_handler.run().await;
    });
    let log_client_clone = log_client.clone();
    tokio::spawn(async move {
        let mut tail_logevent_event_handler = TailLogEventEventHandler::new(
            log_client_clone,
            logevent_state_clone_2_clone,
            tail_logevent_inst_rx_2,
        );
        let _ = tail_logevent_event_handler.run().await;
    });
    let log_client_clone = log_client.clone();
    tokio::spawn(async move {
        let mut tail_logevent_event_handler = TailLogEventEventHandler::new(
            log_client_clone,
            logevent_state_clone_3_clone,
            tail_logevent_inst_rx_3,
        );
        let _ = tail_logevent_event_handler.run().await;
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
