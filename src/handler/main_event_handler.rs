use std::io::Stdout;

use anyhow::Result;
use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;
use tui::{backend::CrosstermBackend, Terminal};

use super::*;
use crate::{app::App, event::Event, terminal::teardown_terminal, ui::Drawable};

pub struct MainEventHandler {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    app: App<CrosstermBackend<Stdout>>,
    input_rx: mpsc::Receiver<Event<KeyEvent>>,
}

impl MainEventHandler {
    pub fn new(
        terminal: Terminal<CrosstermBackend<Stdout>>,
        app: App<CrosstermBackend<Stdout>>,
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
    app: &'a mut App<CrosstermBackend<Stdout>>,
}

impl<'a> Middle<'a> {
    fn new(app: &'a mut App<CrosstermBackend<Stdout>>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl EventHandler for MainEventHandler {
    async fn run(&mut self) -> Result<()> {
        let middle = Middle::new(&mut self.app);
        loop {
            // draw ui according to app state
            self.terminal.draw(|f| middle.app.draw(f, f.size()))?;
            // update app state
            if let Some(event) = self.input_rx.recv().await {
                match event {
                    Event::Input(event) => match event.code {
                        KeyCode::Char('c') => {
                            if let KeyModifiers::CONTROL = event.modifiers {
                                teardown_terminal(&mut self.terminal)?;
                                break;
                            } else {
                                middle.app.handle_event(event).await;
                            }
                        }
                        _ => {
                            middle.app.handle_event(event).await;
                        }
                    },
                    Event::Tick => {}
                }
            }
        }
        Ok(())
    }
}
