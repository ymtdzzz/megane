use std::{
    fmt::{Display, Formatter, Result},
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use chrono::{DateTime, Local, TimeZone};
use crossterm::event::{KeyCode, KeyEvent};
use lazy_static::lazy_static;
use tokio::sync::mpsc;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Row, Table, TableState, Wrap},
    Frame,
};

use crate::{
    constant,
    event::LogEventEvent,
    loader::Loader,
    state::{logevents_state::LogEventsState, search_state::*},
    ui::Drawable,
    utils::get_inner_area,
};

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
        input_cloned.insert_str(self.cursor, "|");
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
                constant::ACTIVE_STYLE.clone()
            } else {
                constant::NORMAL_STYLE.clone()
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
                    self.input.remove(self.cursor - 1);
                    self.cursor_previous();
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
}
