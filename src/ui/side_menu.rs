use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use crossterm::event::KeyEvent;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListState},
    Frame,
};

use crate::{constant, state::loggroups_state::LogGroupsState, ui::Drawable};

pub struct SideMenu<B>
where
    B: Backend,
{
    state: Arc<Mutex<LogGroupsState>>,
    is_selected: bool,
    _phantom: PhantomData<B>,
}

impl<B> SideMenu<B>
where
    B: Backend,
{
    pub fn new(state: Arc<Mutex<LogGroupsState>>) -> Self {
        SideMenu {
            state,
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
            state: Arc::new(Mutex::new(LogGroupsState::new())),
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
        let mut state = self.state.try_lock();
        let (list_items, mut list_state) = match state.as_mut() {
            Ok(s) => s.get_list_items(),
            Err(_) => (vec![], ListState::default()),
        };
        let base_block = Block::default()
            .borders(Borders::ALL)
            .border_style(if self.is_selected {
                Style::default().fg(constant::SELECTED_COLOR.clone())
            } else {
                Style::default().fg(constant::DESELECTED_COLOR.clone())
            })
            .title("Log Groups");
        let list_block = List::new(list_items)
            .block(base_block)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");

        f.render_stateful_widget(list_block, area, &mut list_state);
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
