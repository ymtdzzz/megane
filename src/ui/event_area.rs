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
    log_group_name: String,
    is_selected: bool,
    _phantom: PhantomData<B>,
}

impl<B> EventArea<B>
where
    B: Backend,
{
    pub fn new(log_group_name: String) -> Self {
        EventArea {
            log_group_name,
            is_selected: false,
            _phantom: PhantomData,
        }
    }

    pub fn set_select(&mut self, select: bool) {
        self.is_selected = select;
    }

    pub fn log_group_name(&self) -> &str {
        self.log_group_name.as_str()
    }
}

impl<B> Default for EventArea<B>
where
    B: Backend,
{
    fn default() -> Self {
        EventArea {
            log_group_name: String::from("Events"),
            is_selected: false,
            _phantom: PhantomData,
        }
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
            .title(self.log_group_name.as_ref());

        f.render_widget(block, area);
    }

    async fn handle_event(&mut self, _event: KeyEvent) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyModifiers};
    use tui::{backend::TestBackend, buffer::Buffer, style::Color};

    use super::*;
    use crate::test_helper::get_test_terminal;

    fn test_case(event_area: &mut EventArea<TestBackend>, color: Color, lines: Vec<&str>) {
        let mut terminal = get_test_terminal(20, 10);
        let lines = if lines.len() > 0 {
            lines
        } else {
            vec![
                "┌Events────────────┐",
                "│                  │",
                "│                  │",
                "│                  │",
                "│                  │",
                "│                  │",
                "│                  │",
                "│                  │",
                "│                  │",
                "└──────────────────┘",
            ]
        };
        let mut expected = Buffer::with_lines(lines);
        for y in 0..10 {
            for x in 0..20 {
                let ch = expected.get_mut(x, y);
                if y == 0 || y == 9 {
                    ch.set_fg(color);
                } else {
                    if ch.symbol == "│" {
                        ch.set_fg(color);
                    }
                }
            }
        }
        terminal
            .draw(|f| {
                event_area.draw(f, f.size());
            })
            .unwrap();
        terminal.backend().assert_buffer(&expected);
    }

    #[tokio::test]
    async fn test_draw() {
        let mut event_area: EventArea<TestBackend> = EventArea::default();
        test_case(&mut event_area, Color::White, vec![]);
        event_area.set_select(true);
        test_case(&mut event_area, Color::Yellow, vec![]);
    }

    #[tokio::test]
    async fn test_handle_event() {
        let mut event_area: EventArea<TestBackend> = EventArea::default();
        assert!(
            !event_area
                .handle_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE))
                .await
        );
    }
}
