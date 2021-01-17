use std::{
    fmt::{Display, Formatter, Result},
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
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
    ui::{textbox::TextBox, Drawable},
    utils::get_inner_area,
};

lazy_static! {
    pub static ref MODE_LIST: Vec<SearchMode> = vec![
        SearchMode::Tail,
        SearchMode::OneMinute,
        SearchMode::ThirtyMinutes,
        SearchMode::OneHour,
        SearchMode::TwelveHours,
        SearchMode::FromTo(None, None),
    ];
    pub static ref MODE_NUM: usize = 6;
}

enum CustomInputMode {
    From,
    To,
    None,
}

struct Radio {
    is_selected: bool,
    label: String,
}

impl Radio {
    fn new(label: &str, is_selected: bool) -> Self {
        Radio {
            is_selected,
            label: label.to_string(),
        }
    }

    fn select(&mut self) {
        self.is_selected = true;
    }

    fn deselect(&mut self) {
        self.is_selected = false;
    }
}

impl Display for Radio {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let prefix = if self.is_selected { "[*]" } else { "[ ]" };
        write!(f, "{} {}", prefix, self.label)
    }
}

struct Radios {
    tail: Radio,
    one_minute: Radio,
    thirty_minutes: Radio,
    one_hour: Radio,
    twelve_hourse: Radio,
    custom: Radio,
}

impl Radios {
    fn new() -> Self {
        Radios {
            tail: Radio::new("tail", true),
            one_minute: Radio::new("1 minute", false),
            thirty_minutes: Radio::new("30 minutes", false),
            one_hour: Radio::new("1 hour", false),
            twelve_hourse: Radio::new("12 hours", false),
            custom: Radio::new("custom (from to)", false),
        }
    }

    fn select(&mut self, mode: &SearchMode) {
        self.tail.deselect();
        self.one_minute.deselect();
        self.thirty_minutes.deselect();
        self.one_hour.deselect();
        self.twelve_hourse.deselect();
        self.custom.deselect();
        match mode {
            SearchMode::Tail => {
                self.tail.select();
            }
            SearchMode::OneMinute => {
                self.one_minute.select();
            }
            SearchMode::ThirtyMinutes => {
                self.thirty_minutes.select();
            }
            SearchMode::OneHour => {
                self.one_hour.select();
            }
            SearchMode::TwelveHours => {
                self.twelve_hourse.select();
            }
            SearchMode::FromTo(_, _) => {
                self.custom.select();
            }
        }
    }

    fn get_radio(&self, mode: &SearchMode) -> String {
        match mode {
            SearchMode::Tail => {
                format!("{}", self.tail)
            }
            SearchMode::OneMinute => {
                format!("{}", self.one_minute)
            }
            SearchMode::ThirtyMinutes => {
                format!("{}", self.thirty_minutes)
            }
            SearchMode::OneHour => {
                format!("{}", self.one_hour)
            }
            SearchMode::TwelveHours => {
                format!("{}", self.twelve_hourse)
            }
            SearchMode::FromTo(_, _) => {
                format!("{}", self.custom)
            }
        }
    }
}

pub struct SearchConditionDialog<B>
where
    B: Backend,
{
    focus: usize,
    state: SearchState,
    radios: Radios,
    query_input: TextBox<B>,
    term_mode: CustomInputMode,
    term_from: TextBox<B>,
    term_to: TextBox<B>,
    _phantom: PhantomData<B>,
}

impl<B> SearchConditionDialog<B>
where
    B: Backend,
{
    pub fn new(state: SearchState) -> Self {
        SearchConditionDialog {
            focus: 0,
            state,
            radios: Radios::new(),
            query_input: TextBox::new(true),
            term_mode: CustomInputMode::None,
            term_from: TextBox::new(false),
            term_to: TextBox::new(false),
            _phantom: PhantomData,
        }
    }

    pub fn get_state(&self) -> anyhow::Result<SearchState> {
        let mut s = self.state.clone();
        s.mode = if let SearchMode::FromTo(_, _) = s.mode {
            let (from, to) = self.get_timestamps()?;
            SearchMode::FromTo(from, to)
        } else {
            s.mode
        };
        s.query = self.query_input.get_input();
        Ok(s)
    }

    fn next(&mut self) {
        let max_idx = MODE_NUM.clone() + 1;
        if self.focus < max_idx - 1 {
            self.focus = self.focus.saturating_add(1);
        }
        self.update_input_states();
    }

    fn previous(&mut self) {
        self.focus = self.focus.saturating_sub(1);
        self.update_input_states();
    }

    fn update_input_states(&mut self) {
        // query input state
        if self.focus != 0 {
            self.query_input.deselect();
        } else {
            self.query_input.select();
        }
        // custom term input state
        let list = MODE_LIST.clone();
        let mode = list.get(self.focus.saturating_sub(1));
        if let Some(m) = mode {
            if let SearchMode::FromTo(_, _) = m {
                match self.term_mode {
                    CustomInputMode::From => {
                        self.term_from.select();
                        self.term_to.deselect();
                    }
                    CustomInputMode::To => {
                        self.term_from.deselect();
                        self.term_to.select();
                    }
                    CustomInputMode::None => {
                        self.term_from.deselect();
                        self.term_to.deselect();
                    }
                }
            } else {
                self.term_from.deselect();
                self.term_to.deselect();
            }
        }
    }

    fn select(&mut self) {
        if self.focus == 0 {
            return;
        }
        let list = MODE_LIST.clone();
        let mode = list.get(self.focus.saturating_sub(1));
        if let Some(m) = mode {
            self.radios.select(m);
            self.state.mode = m.clone();
        }
    }

    fn toggle_term_mode(&mut self) {
        match self.term_mode {
            CustomInputMode::From => {
                self.term_mode = CustomInputMode::To;
            }
            CustomInputMode::To => {
                self.term_mode = CustomInputMode::None;
            }
            CustomInputMode::None => {
                self.term_mode = CustomInputMode::From;
            }
        }
    }

    fn get_timestamps(&self) -> anyhow::Result<(Option<i64>, Option<i64>)> {
        let fmt = constant::DATE_FORMAT.clone();
        let from = if self.term_from.get_input().is_empty() {
            None
        } else {
            Some(
                NaiveDateTime::parse_from_str(&self.term_from.get_input(), &fmt)?
                    .timestamp_millis(),
            )
        };
        let to = if self.term_to.get_input().is_empty() {
            None
        } else {
            Some(NaiveDateTime::parse_from_str(&self.term_to.get_input(), &fmt)?.timestamp_millis())
        };
        Ok((from, to))
    }
}

impl<B> Default for SearchConditionDialog<B>
where
    B: Backend,
{
    fn default() -> Self {
        SearchConditionDialog {
            focus: 0,
            state: SearchState::default(),
            radios: Radios::new(),
            query_input: TextBox::default(),
            term_mode: CustomInputMode::From,
            term_from: TextBox::default(),
            term_to: TextBox::default(),
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<B> Drawable<B> for SearchConditionDialog<B>
where
    B: Backend + Send,
{
    fn draw(&mut self, f: &mut Frame<'_, B>, area: Rect) {
        // compute draw area
        let outer_block = Block::default()
            .borders(Borders::ALL)
            .title("Search Condition");
        let outer_area = get_inner_area(&area);
        let inner_area = get_inner_area(&outer_area);
        // prepare inner area
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Length(3),
                    Constraint::Length(1),
                    Constraint::Length(MODE_NUM.clone() as u16),
                ]
                .as_ref(),
            )
            .split(inner_area);

        // input query
        let query_title = Paragraph::new("Query").block(Block::default());
        let query_block = Block::default()
            .borders(Borders::ALL)
            .style(if self.focus == 0 {
                constant::ACTIVE_STYLE.clone()
            } else {
                constant::NORMAL_STYLE.clone()
            });

        // input term
        let radio_areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1),
                    Constraint::Length(1), // custom term input area
                ]
                .as_ref(),
            )
            .split(chunks[3]);
        // check if num of areas for radio buttons is correct
        assert_eq!(MODE_NUM.clone() + 1, radio_areas.len());

        let term_title = Paragraph::new("Term").block(Block::default());

        f.render_widget(outer_block, outer_area);
        f.render_widget(query_title, chunks[0]);
        self.query_input.draw(f, chunks[1]);
        f.render_widget(term_title, chunks[2]);
        MODE_LIST.clone().iter().enumerate().for_each(|(i, v)| {
            let radio = Paragraph::new(self.radios.get_radio(&v)).block(Block::default().style(
                if i + 1 == self.focus {
                    constant::ACTIVE_STYLE.clone()
                } else {
                    constant::NORMAL_STYLE.clone()
                },
            ));
            f.render_widget(radio, radio_areas[i]);
        });
        // custom term input area
        let paragraph = Paragraph::new("~")
            .block(Block::default())
            .style(constant::NORMAL_STYLE.clone());
        let custom_input_areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(50),
                    Constraint::Min(1),
                    Constraint::Percentage(50),
                ]
                .as_ref(),
            )
            .split(*radio_areas.last().unwrap());
        // TODO: The way should be exist to fix height without this line
        let custom_input_areas = custom_input_areas
            .into_iter()
            .map(|mut area| {
                area.height = 3;
                area
            })
            .collect::<Vec<Rect>>();
        self.term_from.draw(f, custom_input_areas[0]);
        f.render_widget(paragraph, custom_input_areas[1]);
        self.term_to.draw(f, custom_input_areas[2]);
    }

    async fn handle_event(&mut self, event: KeyEvent) -> bool {
        if !self.query_input.handle_event(event).await
            && !self.term_from.handle_event(event).await
            && !self.term_to.handle_event(event).await
        {
            match event.code {
                KeyCode::Down => {
                    self.next();
                }
                KeyCode::Up => {
                    self.previous();
                }
                KeyCode::Char(' ') => {
                    self.select();
                }
                KeyCode::Tab => {
                    self.toggle_term_mode();
                    self.update_input_states();
                }
                // events Enter and Esc will be handled by the parent component
                KeyCode::Enter => {
                    return false;
                }
                KeyCode::Esc => {
                    return false;
                }
                _ => {}
            }
        }
        true
    }
}
