use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use chrono::{DateTime, Local, TimeZone};
use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Rect},
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
};

pub struct SearchInfo<B>
where
    B: Backend,
{
    state: SearchState,
    _phantom: PhantomData<B>,
}

impl<B> SearchInfo<B>
where
    B: Backend,
{
    pub fn new(state: SearchState) -> Self {
        SearchInfo {
            state,
            _phantom: PhantomData,
        }
    }

    fn get_msg(&self) -> String {
        format!("query: [{}], mode: [{}]", self.state.query, self.state.mode)
    }

    pub fn set_state(&mut self, new_state: SearchState) {
        self.state = new_state;
    }

    pub fn get_state(&self) -> SearchState {
        self.state.clone()
    }

    pub fn is_same_state(&self, other_state: &SearchState) -> bool {
        &self.state == other_state
    }
}

impl<B> Default for SearchInfo<B>
where
    B: Backend,
{
    fn default() -> Self {
        SearchInfo {
            state: SearchState::default(),
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<B> Drawable<B> for SearchInfo<B>
where
    B: Backend + Send,
{
    fn draw(&mut self, f: &mut Frame<'_, B>, area: Rect) {
        let block = Block::default().borders(Borders::NONE);
        let paragraph = Paragraph::new(self.get_msg())
            .block(block)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
    }

    async fn handle_event(&mut self, event: KeyEvent) -> bool {
        false
    }
}
