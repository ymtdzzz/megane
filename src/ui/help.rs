use std::{
    marker::PhantomData,
};

use async_trait::async_trait;
use crossterm::event::{KeyEvent};
use tui::{
    backend::Backend,
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::{constant, ui::Drawable};

pub struct Help<B>
where
    B: Backend,
{
    msg: String,
    _phantom: PhantomData<B>,
}

impl<B> Help<B>
where
    B: Backend,
{
    pub fn new() -> Self {
        Help {
            msg: constant::HELP_MESSAGE.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<B> Default for Help<B>
where
    B: Backend,
{
    fn default() -> Self {
        Help {
            msg: constant::HELP_MESSAGE.clone(),
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<B> Drawable<B> for Help<B>
where
    B: Backend + Send,
{
    fn draw(&mut self, f: &mut Frame<B>, area: Rect) {
        let block = Block::default()
            .title("HELP".to_string())
            .borders(Borders::ALL);
        let paragraph = Paragraph::new(self.msg.as_ref())
            .block(block)
            .wrap(Wrap { trim: false });
        f.render_widget(paragraph, area);
    }

    async fn handle_event(&mut self, _event: KeyEvent) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {}
