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

pub struct SideMenu<B>
where
    B: Backend,
{
    is_selected: bool,
    _phantom: PhantomData<B>,
}

impl<B> SideMenu<B>
where
    B: Backend,
{
    pub fn new() -> Self {
        SideMenu {
            is_selected: true,
            _phantom: PhantomData,
        }
    }

    pub fn set_select(&mut self, select: bool) {
        self.is_selected = select;
    }
}

impl<B> Default for SideMenu<B>
where
    B: Backend,
{
    fn default() -> Self {
        SideMenu {
            is_selected: false,
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<B> Drawable<B> for SideMenu<B>
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
            .title("Log Groups");

        f.render_widget(block, area);
    }

    async fn handle_event(&mut self, _event: KeyEvent) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyModifiers};
    use tui::{backend::TestBackend, buffer::Buffer, style::Color, Terminal};

    use super::*;
    use crate::test_helper::get_test_terminal;

    fn test_case(side_menu: &mut SideMenu<TestBackend>, color: Color, lines: Vec<&str>) {
        let mut terminal = get_test_terminal(20, 10);
        let lines = if lines.len() > 0 {
            lines
        } else {
            vec![
                "┌Log Groups────────┐",
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
                if x == 4 && y == 0 {
                    ch.set_fg(color);
                } else {
                    if ch.symbol != " " {
                        ch.set_fg(color);
                    }
                }
            }
        }
        terminal
            .draw(|f| {
                side_menu.draw(f, f.size());
            })
            .unwrap();
        terminal.backend().assert_buffer(&expected);
    }

    #[tokio::test]
    async fn test_draw() {
        let mut side_menu: SideMenu<TestBackend> = SideMenu::default();
        test_case(&mut side_menu, Color::White, vec![]);
        side_menu.set_select(true);
        test_case(&mut side_menu, Color::Yellow, vec![]);
    }

    #[tokio::test]
    async fn test_handle_event() {
        let mut side_menu: SideMenu<TestBackend> = SideMenu::default();
        assert!(
            !side_menu
                .handle_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE))
                .await
        );
    }
}
