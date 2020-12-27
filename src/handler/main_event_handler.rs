use std::io::Stdout;

use anyhow::Result;
use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc;
use tui::{backend::CrosstermBackend, layout::Rect, Terminal};

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
            self.terminal
                .draw(|f| middle.app.draw(f, Rect::default()))?;
            // update app state
            match self.input_rx.recv().await.unwrap() {
                Event::Input(event) => match event.code {
                    KeyCode::Char('q') => {
                        teardown_terminal(&mut self.terminal)?;
                        break;
                    }
                    // TODO: delegate handling key input to app
                    _ => {
                        middle.app.handle_event(event).await;
                    }
                },
                Event::Tick => {}
            }
        }
        Ok(())
    }
}
