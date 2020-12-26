use std::io::{Stdout, Write};

use anyhow::Result;
use async_trait::async_trait;
use crossterm::{
    event::{DisableMouseCapture, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use tokio::sync::mpsc;
use tui::{backend::CrosstermBackend, Terminal};

use super::*;
use crate::{app::App, event::Event, ui};

pub struct MainEventHandler {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    app: App,
    input_rx: mpsc::Receiver<Event<KeyEvent>>,
}

impl MainEventHandler {
    pub fn new(
        terminal: Terminal<CrosstermBackend<Stdout>>,
        app: App,
        input_rx: mpsc::Receiver<Event<KeyEvent>>,
    ) -> Self {
        MainEventHandler {
            terminal,
            app,
            input_rx,
        }
    }
}

struct Middle<'a> {
    app: &'a mut App,
}

impl<'a> Middle<'a> {
    fn new(app: &'a mut App) -> Self {
        Self { app }
    }
}

#[async_trait]
impl EventHandler for MainEventHandler {
    async fn run(&mut self) -> Result<()> {
        let mut middle = Middle::new(&mut self.app);
        loop {
            // draw ui according to app state
            self.terminal.draw(|f| ui::draw(f, &mut middle.app))?;
            // update app state
            match self.input_rx.recv().await.unwrap() {
                Event::Input(event) => match event.code {
                    KeyCode::Char('q') => {
                        // quit
                        disable_raw_mode()?;
                        let backend = self.terminal.backend_mut();
                        execute!(backend, LeaveAlternateScreen, DisableMouseCapture)?;
                        self.terminal.show_cursor()?;
                        break;
                    }
                    // TODO: delegate handling key input to app
                    _ => {}
                },
                Event::Tick => {}
            }
        }
        Ok(())
    }
}
