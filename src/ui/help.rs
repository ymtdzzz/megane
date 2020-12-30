use std::marker::PhantomData;

use async_trait::async_trait;
use crossterm::event::KeyEvent;
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
            msg: String::from(""),
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<B> Drawable<B> for Help<B>
where
    B: Backend + Send,
{
    fn draw(&mut self, f: &mut Frame<'_, B>, area: Rect) {
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
mod tests {

    use tui::{backend::TestBackend, buffer::Buffer};

    use super::*;
    use crate::test_helper::get_test_terminal;

    fn test_case(help: &mut Help<TestBackend>, lines: Vec<&str>) {
        let mut terminal = get_test_terminal(20, 10);
        let lines = if !lines.is_empty() {
            lines
        } else {
            vec![
                "┌HELP──────────────┐",
                "│                  │",
                "│test message      │",
                "│12345             │",
                "│                  │",
                "│                  │",
                "│                  │",
                "│                  │",
                "│                  │",
                "└──────────────────┘",
            ]
        };
        let expected = Buffer::with_lines(lines);
        terminal
            .draw(|f| {
                help.draw(f, f.size());
            })
            .unwrap();
        terminal.backend().assert_buffer(&expected);
    }

    #[tokio::test]
    async fn test_draw() {
        let mut help: Help<TestBackend> = Help::default();
        help.msg = String::from(
            r#"
test message
12345
        "#,
        );
        test_case(&mut help, vec![]);
    }
}
