use std::io::Stdout;

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::Block,
    Frame,
};

use crate::ui::Drawable;

/// which component selected
enum SelectState {
    SideMenu,
    EventAreas(usize),
}

pub struct App {
    side_menu: Box<dyn Drawable + Send>,
    event_areas: Vec<Box<dyn Drawable + Send>>,
    select_state: SelectState,
}

impl App {
    pub async fn new(
        side_menu: Box<dyn Drawable + Send>,
        event_areas: Vec<Box<dyn Drawable + Send>>,
    ) -> App {
        App {
            side_menu,
            event_areas,
            select_state: SelectState::SideMenu,
        }
    }
}

#[async_trait]
impl Drawable for App {
    fn draw(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>, _area: Rect) {
        let chunks = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(f.size());
        self.side_menu.draw(f, chunks[0]);
        // TODO: log event area
    }

    async fn handle_event(&mut self, event: KeyEvent) -> bool {
        let solved = match self.select_state {
            SelectState::SideMenu => self.side_menu.handle_event(event).await,
            SelectState::EventAreas(idx) => {
                if let Some(logarea) = self.event_areas.get_mut(idx) {
                    logarea.handle_event(event).await
                } else {
                    false
                }
            }
        };
        if !solved {
            match event.code {
                KeyCode::Tab => {
                    // TODO: toggle collapse side menu
                }
                _ => {}
            }
        }
        true
    }
}
