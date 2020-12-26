use anyhow::Result;
use async_trait::async_trait;
use crossterm::{
    event::{DisableMouseCapture, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use tokio::sync::mpsc;
use tui::{backend::Backend, layout::Layout, Terminal};

use super::*;
use crate::{app::App, event::Event, ui::Drawable};

pub struct MainEventHandler<B>
where
    B: Backend,
{
    terminal: Terminal<B>,
    app: App<B>,
    input_rx: mpsc::Receiver<Event<KeyEvent>>,
}

impl<B> MainEventHandler<B>
where
    B: Backend,
{
    pub fn new(
        terminal: Terminal<B>,
        app: App<B>,
        input_rx: mpsc::Receiver<Event<KeyEvent>>,
    ) -> Self {
        MainEventHandler {
            terminal,
            app,
            input_rx,
        }
    }
}

struct Middle<'a, B: 'a>
where
    B: Backend,
{
    app: &'a mut App<B>,
}

impl<'a, B: 'a> Middle<'a, B>
where
    B: Backend,
{
    fn new(app: &'a mut App<B>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl<B: std::fmt::Write> EventHandler for MainEventHandler<B>
where
    B: Backend + Send,
{
    async fn run(&mut self) -> Result<()> {
        let middle = Middle::new(&mut self.app);
        loop {
            // draw ui according to app state
            self.terminal
                .draw(|f| middle.app.draw(f, Layout::default().split(f.size())[0]))?;
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
