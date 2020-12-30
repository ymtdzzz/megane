use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use crossterm::event::KeyEvent;
use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::{state::status_bar_state::StatusBarState, ui::Drawable};

pub struct StatusBar<B>
where
    B: Backend,
{
    state: Arc<Mutex<StatusBarState>>,
    _phantom: PhantomData<B>,
}

impl<B> StatusBar<B>
where
    B: Backend,
{
    pub fn new(state: Arc<Mutex<StatusBarState>>) -> Self {
        StatusBar {
            state,
            _phantom: PhantomData,
        }
    }
}

impl<B> Default for StatusBar<B>
where
    B: Backend,
{
    fn default() -> Self {
        StatusBar {
            state: Arc::new(Mutex::new(StatusBarState::new(String::from(
                "initial message",
            )))),
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<B> Drawable<B> for StatusBar<B>
where
    B: Backend + Send,
{
    fn draw(&mut self, f: &mut Frame<'_, B>, area: Rect) {
        let state = self.state.try_lock();
        let message = match state.as_ref() {
            Ok(s) => s.message.as_str(),
            Err(_) => "",
        };
        let block = Block::default().borders(Borders::NONE);
        let paragraph = Paragraph::new(message)
            .block(block)
            .alignment(Alignment::Right)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
    }

    async fn handle_event(&mut self, _event: KeyEvent) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyModifiers};
    use tui::{backend::TestBackend, buffer::Buffer};

    use super::*;
    use crate::test_helper::get_test_terminal;

    fn test_case(status_bar: &mut StatusBar<TestBackend>, lines: Vec<&str>) {
        let mut terminal = get_test_terminal(20, 4);
        let lines = if !lines.is_empty() {
            lines
        } else {
            vec![
                "     initial message",
                "                    ",
                "                    ",
                "                    ",
            ]
        };
        let expected = Buffer::with_lines(lines);
        terminal
            .draw(|f| {
                status_bar.draw(f, f.size());
            })
            .unwrap();
        terminal.backend().assert_buffer(&expected);
    }

    #[test]
    fn test_draw() {
        let mut status_bar: StatusBar<TestBackend> = StatusBar::default();
        test_case(&mut status_bar, vec![]);
        status_bar.state.lock().unwrap().message = String::from("1234567890123456789012345");
        let lines = vec![
            "12345678901234567890",
            "               12345",
            "                    ",
            "                    ",
        ];
        test_case(&mut status_bar, lines);
    }

    #[tokio::test]
    async fn test_handle_event() {
        let mut status_bar: StatusBar<TestBackend> = StatusBar::default();
        assert!(
            !status_bar
                .handle_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
                .await
        );
    }
}
