use std::marker::PhantomData;

use async_trait::async_trait;
use crossterm::event::KeyEvent;
use tui::{
    backend::Backend,
    layout::Rect,
    style::Style,
    widgets::{Block, Borders},
    Frame,
};

use crate::{constant, ui::Drawable};

pub struct EventArea<B>
where
    B: Backend,
{
    is_selected: bool,
    _phantom: PhantomData<B>,
}

impl<B> EventArea<B>
where
    B: Backend,
{
    pub fn new() -> Self {
        EventArea {
            is_selected: false,
            _phantom: PhantomData,
        }
    }

    pub fn set_select(&mut self, select: bool) {
        self.is_selected = select;
    }
}

#[async_trait]
impl<B> Drawable<B> for EventArea<B>
where
    B: Backend + Send,
{
    fn draw(&mut self, f: &mut Frame<B>, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(if self.is_selected {
                Style::default().fg(constant::SELECTED_COLOR.clone())
            } else {
                Style::default().fg(constant::DESELECTED_COLOR.clone())
            })
            .title("Events");

        f.render_widget(block, area);
    }

    async fn handle_event(&mut self, _event: KeyEvent) -> bool {
        false
    }
}
