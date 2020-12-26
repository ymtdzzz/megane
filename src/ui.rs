use async_trait::async_trait;
use crossterm::event::KeyEvent;
use tui::{backend::Backend, layout::Rect, Frame};

mod side_menu;

#[async_trait]
pub trait Drawable<B>
where
    B: Backend,
{
    /// all components must be drawable
    fn draw(&mut self, f: &mut Frame<B>, area: Rect);

    /// handles input key event
    /// and returns if parent component should handle other events or not
    async fn handle_event(&mut self, event: KeyEvent) -> bool;
}
