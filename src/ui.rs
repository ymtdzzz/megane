use std::io::Stdout;

use async_trait::async_trait;
use crossterm::event::KeyEvent;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::Block,
    Frame,
};

use super::app::App;

mod side_menu;

#[async_trait]
pub trait Drawable {
    /// all components must be drawable
    fn draw(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>, area: Rect);

    /// handles input key event
    /// and returns if parent component should handle other events or not
    async fn handle_event(&mut self, event: KeyEvent) -> bool;
}
