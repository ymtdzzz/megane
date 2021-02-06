use std::{collections::BTreeMap, marker::PhantomData};

use async_trait::async_trait;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tui::{
    backend::Backend,
    layout::Rect,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{constant, key_event_wrapper::KeyEventWrapper, ui::Drawable};

pub struct TextBox<B>
where
    B: Backend,
{
    is_selected: bool,
    cursor: usize,
    input: String,
    _phantom: PhantomData<B>,
}

impl<B> TextBox<B>
where
    B: Backend,
{
    pub fn new(is_selected: bool) -> Self {
        TextBox {
            is_selected,
            cursor: 0,
            input: String::default(),
            _phantom: PhantomData,
        }
    }

    pub fn get_input(&self) -> String {
        self.input.clone()
    }

    fn get_text_to_show(&self) -> String {
        let mut input_cloned = self.input.clone();
        input_cloned.insert(self.cursor, '|');
        input_cloned
    }

    fn cursor_next(&mut self) {
        if self.cursor < self.input.len() {
            self.cursor = self.cursor.saturating_add(1);
        }
    }

    fn cursor_previous(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn select(&mut self) {
        self.is_selected = true;
    }

    pub fn deselect(&mut self) {
        self.is_selected = false;
    }
}

impl<B> Default for TextBox<B>
where
    B: Backend,
{
    fn default() -> Self {
        TextBox {
            is_selected: false,
            cursor: 0,
            input: String::default(),
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<B> Drawable<B> for TextBox<B>
where
    B: Backend + Send,
{
    fn draw(&mut self, f: &mut Frame<'_, B>, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .style(if self.is_selected {
                *constant::ACTIVE_STYLE
            } else {
                *constant::NORMAL_STYLE
            });
        let paragraph = Paragraph::new(self.get_text_to_show()).block(block);
        f.render_widget(paragraph, area);
    }

    async fn handle_event(&mut self, event: KeyEvent) -> bool {
        if self.is_selected {
            match event.code {
                KeyCode::Left => {
                    self.cursor_previous();
                }
                KeyCode::Right => {
                    self.cursor_next();
                }
                KeyCode::Char(c) => {
                    self.input.insert(self.cursor, c);
                    self.cursor_next();
                }
                KeyCode::Backspace => {
                    if self.cursor > 0 {
                        self.input.remove(self.cursor.saturating_sub(1));
                        self.cursor_previous();
                    }
                }
                _ => {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }

    fn push_key_maps<'a>(
        &self,
        maps: &'a mut BTreeMap<KeyEventWrapper, String>,
    ) -> &'a mut BTreeMap<KeyEventWrapper, String> {
        if self.is_selected {
            maps.insert(
                KeyEventWrapper::new(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE)),
                "Move cursor".to_string(),
            );
            maps.insert(
                KeyEventWrapper::new(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE)),
                "Move cursor".to_string(),
            );
            maps.insert(
                KeyEventWrapper::new(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)),
                "Delete char".to_string(),
            );
        }
        maps
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyModifiers};
    use tui::{backend::TestBackend, buffer::Buffer, style::Color};

    use super::*;
    use crate::test_helper::{get_test_terminal, key_maps_test_case};

    fn test_case(textbox: &mut TextBox<TestBackend>, expected: Buffer) {
        let mut terminal = get_test_terminal(20, 3);
        terminal
            .draw(|f| {
                textbox.draw(f, f.size());
            })
            .unwrap();
        terminal.backend().assert_buffer(&expected);
    }

    #[test]
    fn test_draw() {
        let mut textbox: TextBox<TestBackend> = TextBox::new(false);
        let lines = vec![
            "┌──────────────────┐",
            "│|input            │",
            "└──────────────────┘",
        ];
        textbox.input = "input".to_string();
        let mut expected = Buffer::with_lines(lines.clone());
        // not selected
        for y in 0..3 {
            for x in 0..20 {
                let ch = expected.get_mut(x, y);
                ch.set_fg(Color::White);
            }
        }
        test_case(&mut textbox, expected);
        // selected
        let mut expected = Buffer::with_lines(lines.clone());
        textbox.is_selected = true;
        for y in 0..3 {
            for x in 0..20 {
                let ch = expected.get_mut(x, y);
                ch.set_fg(Color::Yellow);
            }
        }
        test_case(&mut textbox, expected);
        // overflow check
        textbox.cursor_previous();
        let mut expected = Buffer::with_lines(lines.clone());
        for y in 0..3 {
            for x in 0..20 {
                let ch = expected.get_mut(x, y);
                ch.set_fg(Color::Yellow);
            }
        }
        test_case(&mut textbox, expected);
        let lines = vec![
            "┌──────────────────┐",
            "│input|            │",
            "└──────────────────┘",
        ];
        textbox.cursor = 5;
        textbox.cursor_next();
        let mut expected = Buffer::with_lines(lines.clone());
        for y in 0..3 {
            for x in 0..20 {
                let ch = expected.get_mut(x, y);
                ch.set_fg(Color::Yellow);
            }
        }
        test_case(&mut textbox, expected);
        // cursor previous
        let lines = vec![
            "┌──────────────────┐",
            "│inpu|t            │",
            "└──────────────────┘",
        ];
        textbox.cursor_previous();
        let mut expected = Buffer::with_lines(lines.clone());
        for y in 0..3 {
            for x in 0..20 {
                let ch = expected.get_mut(x, y);
                ch.set_fg(Color::Yellow);
            }
        }
        test_case(&mut textbox, expected);
        // cursor next
        let lines = vec![
            "┌──────────────────┐",
            "│input|            │",
            "└──────────────────┘",
        ];
        textbox.cursor_next();
        let mut expected = Buffer::with_lines(lines.clone());
        for y in 0..3 {
            for x in 0..20 {
                let ch = expected.get_mut(x, y);
                ch.set_fg(Color::Yellow);
            }
        }
        test_case(&mut textbox, expected);
    }

    #[tokio::test]
    async fn test_handle_event() {
        let mut textbox: TextBox<TestBackend> = TextBox::new(true);
        textbox.input = "input".to_string();
        assert_eq!(0, textbox.cursor);
        assert!(
            textbox
                .handle_event(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE))
                .await
        );
        assert_eq!(1, textbox.cursor);
        assert!(
            textbox
                .handle_event(KeyEvent::new(KeyCode::Left, KeyModifiers::NONE))
                .await
        );
        assert_eq!(0, textbox.cursor);
        textbox.cursor = 4;
        assert!(
            textbox
                .handle_event(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE))
                .await
        );
        assert_eq!("inpt".to_string(), textbox.get_input());
        assert_eq!(3, textbox.cursor);
    }

    #[test]
    fn test_push_key_maps() {
        // selected
        let textbox: TextBox<TestBackend> = TextBox::new(true);
        key_maps_test_case(
            &textbox,
            KeyEvent::new(KeyCode::Left, KeyModifiers::NONE),
            "Move cursor",
        );
        key_maps_test_case(
            &textbox,
            KeyEvent::new(KeyCode::Right, KeyModifiers::NONE),
            "Move cursor",
        );
        key_maps_test_case(
            &textbox,
            KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
            "Delete char",
        );
        // not selected
        let textbox: TextBox<TestBackend> = TextBox::new(false);
        let mut maps: BTreeMap<KeyEventWrapper, String> = BTreeMap::new();
        textbox.push_key_maps(&mut maps);
        assert!(maps.is_empty())
    }
}
