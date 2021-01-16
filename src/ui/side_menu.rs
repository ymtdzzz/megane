use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent};
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::{constant, loader::Loader, state::loggroups_state::LogGroupsState, ui::Drawable};

pub struct SideMenu<B>
where
    B: Backend,
{
    state: Arc<Mutex<LogGroupsState>>,
    is_selected: bool,
    selected_log_groups: Vec<String>,
    loader: Loader,
    query: String,
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
            selected_log_groups: vec![],
            loader: Loader::new(constant::LOADER.clone()),
            query: String::from(""),
            _phantom: PhantomData,
        }
    }

    pub fn set_select(&mut self, select: bool) {
        self.is_selected = select;
    }

    pub fn selected_log_groups(&self) -> &Vec<String> {
        self.selected_log_groups.as_ref()
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
            selected_log_groups: vec![],
            loader: Loader::new(constant::LOADER.clone()),
            query: String::from(""),
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<B> Drawable<B> for SideMenu<B>
where
    B: Backend + Send,
{
    fn draw(&mut self, f: &mut Frame<'_, B>, area: Rect) {
        let mut state = self.state.try_lock();
        let (list_items, mut list_state) = match state.as_mut() {
            Ok(s) => {
                let is_fetching = s.is_fetching;
                let (mut items, state) = s.get_list_items(&self.query, &self.selected_log_groups);
                if is_fetching {
                    items.push(ListItem::new(self.loader.get_char().to_string()));
                }
                (items, state)
            }
            Err(_) => (vec![], ListState::default()),
        };
        let base_block = Block::default()
            .borders(Borders::ALL)
            .border_style(if self.is_selected {
                Style::default().fg(*constant::SELECTED_COLOR)
            } else {
                Style::default().fg(*constant::DESELECTED_COLOR)
            })
            .title(format!(
                "Log Groups [{}]",
                if self.query.is_empty() {
                    "type to search"
                } else {
                    &self.query
                }
            ));
        let list_block = List::new(list_items)
            .block(base_block)
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");

        f.render_stateful_widget(list_block, area, &mut list_state);
    }

    async fn handle_event(&mut self, event: KeyEvent) -> bool {
        if self.is_selected {
            let mut state = self.state.try_lock();
            match event.code {
                KeyCode::Char(c) => {
                    if c != '?' {
                        // ? is the key to toggle help dialog
                        self.query.push(c);
                    }
                }
                KeyCode::Backspace => {
                    self.query.pop();
                }
                KeyCode::Down => {
                    if let Ok(s) = state.as_mut() {
                        s.next();
                    }
                }
                KeyCode::Up => {
                    if let Ok(s) = state.as_mut() {
                        s.previous();
                    }
                }
                KeyCode::Enter => {
                    if let Ok(s) = state.as_mut() {
                        if let Some(idx) = s.get_current_idx() {
                            s.select(idx);
                            self.selected_log_groups = s.get_selected_log_group_names();
                        }
                    }
                }
                _ => {}
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyModifiers};
    use tui::{backend::TestBackend, buffer::Buffer, style::Color};

    use super::*;
    use crate::{loggroups::LogGroups, state::loggroups_state::LogGroupsState, test_helper::*};

    fn test_case(side_menu: &mut SideMenu<TestBackend>, color: Color, lines: Vec<&str>) {
        let mut terminal = get_test_terminal(20, 10);
        let lines = if !lines.is_empty() {
            lines
        } else {
            vec![
                "┌Log Groups [type t┐",
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
                } else if ch.symbol == "│" {
                    ch.set_fg(color);
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
        side_menu.state.lock().unwrap().is_fetching = true;
        let lines = vec![
            "┌Log Groups [type t┐",
            "│⣾                 │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "│                  │",
            "└──────────────────┘",
        ];
        test_case(&mut side_menu, Color::Yellow, lines);
    }

    #[tokio::test]
    async fn test_handle_event() {
        let mut side_menu: SideMenu<TestBackend> = SideMenu::default();
        side_menu.is_selected = true;
        let state = Arc::new(Mutex::new(LogGroupsState::new()));
        state.lock().unwrap().log_groups = LogGroups::new(get_log_groups(0, 3, false));
        state.lock().unwrap().next();
        side_menu.state = Arc::clone(&state);
        assert!(
            !side_menu
                .handle_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE))
                .await
        );
        assert!(
            !side_menu
                .handle_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE))
                .await
        );
        side_menu.state.lock().unwrap().state.select(Some(1));
        assert!(
            !side_menu
                .handle_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
                .await
        );
        assert!(
            !side_menu
                .handle_event(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE))
                .await
        );
        assert!(
            !side_menu
                .handle_event(KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE))
                .await
        );
        assert_eq!(String::from("ab"), side_menu.query);
        assert!(
            !side_menu
                .handle_event(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE))
                .await
        );
        assert_eq!(String::from("a"), side_menu.query);
    }
}
