use std::{
    fmt::{Display, Formatter, Result},
    marker::PhantomData,
};

use async_trait::async_trait;
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use crossterm::event::{KeyCode, KeyEvent};
use lazy_static::lazy_static;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{
    constant,
    state::search_state::*,
    ui::{textbox::TextBox, Drawable},
    utils::*,
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

#[derive(Debug, PartialEq)]
enum CustomInputMode {
    From,
    To,
    None,
}

#[derive(Debug, PartialEq)]
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
        let max_idx = *MODE_NUM + 1;
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
            println!("{:?}", m);
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
        let fmt = &*constant::DATE_FORMAT;
        let from = if self.term_from.get_input().is_empty() {
            None
        } else {
            let naive = NaiveDateTime::parse_from_str(&self.term_from.get_input(), &fmt)?;
            let local = Local.from_local_datetime(&naive).unwrap();
            let utc: DateTime<Utc> = DateTime::from(local);
            Some(utc.timestamp_millis())
        };
        let to = if self.term_to.get_input().is_empty() {
            None
        } else {
            let naive = NaiveDateTime::parse_from_str(&self.term_to.get_input(), &fmt)?;
            let local = Local.from_local_datetime(&naive).unwrap();
            let utc: DateTime<Utc> = DateTime::from(local);
            Some(utc.timestamp_millis())
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
                    Constraint::Length(*MODE_NUM as u16),
                ]
                .as_ref(),
            )
            .split(inner_area);

        // input query
        let query_title = Paragraph::new("Query").block(Block::default());

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
        assert_eq!(*MODE_NUM + 1, radio_areas.len());

        let term_title = Paragraph::new("Term").block(Block::default());

        f.render_widget(outer_block, outer_area);
        f.render_widget(query_title, chunks[0]);
        self.query_input.draw(f, chunks[1]);
        f.render_widget(term_title, chunks[2]);
        MODE_LIST.clone().iter().enumerate().for_each(|(i, v)| {
            let radio = Paragraph::new(self.radios.get_radio(&v)).block(Block::default().style(
                if i + 1 == self.focus {
                    *constant::ACTIVE_STYLE
                } else {
                    *constant::NORMAL_STYLE
                },
            ));
            f.render_widget(radio, radio_areas[i]);
        });
        // custom term input area
        let paragraph = Paragraph::new("~")
            .block(Block::default())
            .style(*constant::NORMAL_STYLE);
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

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyModifiers};
    use tui::{backend::TestBackend, buffer::Buffer, style::Color};

    use super::*;
    use crate::test_helper::get_test_terminal;

    #[test]
    fn test_radio_select() {
        let mut radio = Radio::new("test_radio", false);
        assert!(!radio.is_selected);
        radio.select();
        assert!(radio.is_selected);
        radio.deselect();
        assert!(!radio.is_selected);
    }

    #[test]
    fn test_radio_fmt() {
        let radio_1 = Radio::new("radio_1", false);
        let radio_2 = Radio::new("radio_2", true);
        assert_eq!(String::from("[ ] radio_1"), radio_1.to_string());
        assert_eq!(String::from("[*] radio_2"), radio_2.to_string());
    }

    #[test]
    fn test_radios_select() {
        let mut radios = Radios::new();
        // initial state check
        assert!(radios.tail.is_selected);
        assert!(!radios.one_minute.is_selected);
        assert!(!radios.thirty_minutes.is_selected);
        assert!(!radios.one_hour.is_selected);
        assert!(!radios.twelve_hourse.is_selected);
        assert!(!radios.custom.is_selected);
        // select
        radios.select(&SearchMode::OneMinute);
        assert!(!radios.tail.is_selected);
        assert!(radios.one_minute.is_selected);
        assert!(!radios.thirty_minutes.is_selected);
        assert!(!radios.one_hour.is_selected);
        assert!(!radios.twelve_hourse.is_selected);
        assert!(!radios.custom.is_selected);
        radios.select(&SearchMode::ThirtyMinutes);
        assert!(!radios.tail.is_selected);
        assert!(!radios.one_minute.is_selected);
        assert!(radios.thirty_minutes.is_selected);
        assert!(!radios.one_hour.is_selected);
        assert!(!radios.twelve_hourse.is_selected);
        assert!(!radios.custom.is_selected);
        radios.select(&SearchMode::OneHour);
        assert!(!radios.tail.is_selected);
        assert!(!radios.one_minute.is_selected);
        assert!(!radios.thirty_minutes.is_selected);
        assert!(radios.one_hour.is_selected);
        assert!(!radios.twelve_hourse.is_selected);
        assert!(!radios.custom.is_selected);
        radios.select(&SearchMode::TwelveHours);
        assert!(!radios.tail.is_selected);
        assert!(!radios.one_minute.is_selected);
        assert!(!radios.thirty_minutes.is_selected);
        assert!(!radios.one_hour.is_selected);
        assert!(radios.twelve_hourse.is_selected);
        assert!(!radios.custom.is_selected);
        radios.select(&SearchMode::FromTo(None, None));
        assert!(!radios.tail.is_selected);
        assert!(!radios.one_minute.is_selected);
        assert!(!radios.thirty_minutes.is_selected);
        assert!(!radios.one_hour.is_selected);
        assert!(!radios.twelve_hourse.is_selected);
        assert!(radios.custom.is_selected);
        radios.select(&SearchMode::Tail);
        assert!(radios.tail.is_selected);
        assert!(!radios.one_minute.is_selected);
        assert!(!radios.thirty_minutes.is_selected);
        assert!(!radios.one_hour.is_selected);
        assert!(!radios.twelve_hourse.is_selected);
        assert!(!radios.custom.is_selected);
    }

    #[test]
    fn test_radios_get_radio() {
        let radios = Radios::new();
        assert_eq!("[*] tail".to_string(), radios.get_radio(&SearchMode::Tail));
        assert_eq!(
            "[ ] 1 minute".to_string(),
            radios.get_radio(&SearchMode::OneMinute)
        );
        assert_eq!(
            "[ ] 30 minutes".to_string(),
            radios.get_radio(&SearchMode::ThirtyMinutes)
        );
        assert_eq!(
            "[ ] 1 hour".to_string(),
            radios.get_radio(&SearchMode::OneHour)
        );
        assert_eq!(
            "[ ] 12 hours".to_string(),
            radios.get_radio(&SearchMode::TwelveHours)
        );
        assert_eq!(
            "[ ] custom (from to)".to_string(),
            radios.get_radio(&SearchMode::FromTo(None, None))
        );
    }

    #[test]
    fn test_dialog_get_state() {
        let mut dialog: SearchConditionDialog<TestBackend> =
            SearchConditionDialog::new(SearchState::default());
        let expected = SearchState::default();
        assert_eq!(expected, dialog.get_state().unwrap());
        // custom term
        let expected = SearchState::new(String::default(), SearchMode::FromTo(None, None));
        dialog.focus = 6;
        dialog.select();
        dialog.radios.select(&SearchMode::FromTo(None, None));
        assert_eq!(expected, dialog.get_state().unwrap());
    }

    #[test]
    fn test_dialog_next() {
        let mut dialog: SearchConditionDialog<TestBackend> =
            SearchConditionDialog::new(SearchState::default());
        assert_eq!(0, dialog.focus);
        dialog.next();
        assert_eq!(1, dialog.focus);
        dialog.focus = 6;
        dialog.next();
        assert_eq!(6, dialog.focus);
    }

    #[test]
    fn test_dialog_previous() {
        let mut dialog: SearchConditionDialog<TestBackend> =
            SearchConditionDialog::new(SearchState::default());
        dialog.focus = 6;
        dialog.previous();
        assert_eq!(5, dialog.focus);
        dialog.focus = 0;
        dialog.previous();
        assert_eq!(0, dialog.focus);
    }

    #[test]
    fn test_dialog_select() {
        let mut dialog: SearchConditionDialog<TestBackend> =
            SearchConditionDialog::new(SearchState::default());
        let mut expected: SearchConditionDialog<TestBackend> =
            SearchConditionDialog::new(SearchState::default());
        // no change
        dialog.select();
        assert_eq!(expected.focus, dialog.focus);
        assert_eq!(expected.state, dialog.state);
        // change mode to ThirtyMinutes and select the correct radio
        dialog.focus = 3;
        dialog.select();
        expected.focus = 3;
        expected.state.mode = SearchMode::ThirtyMinutes;
        assert_eq!(expected.focus, dialog.focus);
        assert_eq!(expected.state, dialog.state);
        assert!(dialog.radios.thirty_minutes.is_selected);
    }

    #[test]
    fn test_dialog_toggle_term_mode() {
        let mut dialog: SearchConditionDialog<TestBackend> =
            SearchConditionDialog::new(SearchState::default());
        dialog.term_mode = CustomInputMode::From;
        dialog.toggle_term_mode();
        assert_eq!(CustomInputMode::To, dialog.term_mode);
        dialog.toggle_term_mode();
        assert_eq!(CustomInputMode::None, dialog.term_mode);
        dialog.toggle_term_mode();
        assert_eq!(CustomInputMode::From, dialog.term_mode);
    }

    fn test_case(dialog: &mut SearchConditionDialog<TestBackend>, expected: Buffer) {
        let mut terminal = get_test_terminal(50, 20);
        terminal
            .draw(|f| {
                dialog.draw(f, f.size());
            })
            .unwrap();
        terminal.backend().assert_buffer(&expected);
    }

    #[test]
    fn test_dialog_draw() {
        let mut dialog: SearchConditionDialog<TestBackend> =
            SearchConditionDialog::new(SearchState::default());
        let lines = vec![
            "                                                  ",
            " ┌Search Condition──────────────────────────────┐ ",
            " │Query                                         │ ",
            " │┌────────────────────────────────────────────┐│ ",
            " ││|                                           ││ ",
            " │└────────────────────────────────────────────┘│ ",
            " │Term                                          │ ",
            " │[*] tail                                      │ ",
            " │[ ] 1 minute                                  │ ",
            " │[ ] 30 minutes                                │ ",
            " │[ ] 1 hour                                    │ ",
            " │[ ] 12 hours                                  │ ",
            " │[ ] custom (from to)                          │ ",
            " │┌─────────────────────┐~┌────────────────────┐│ ",
            " ││|                    │ │|                   ││ ",
            " │└─────────────────────┘ └────────────────────┘│ ",
            " │                                              │ ",
            " │                                              │ ",
            " └──────────────────────────────────────────────┘ ",
            "                                                  ",
        ];
        // initial dialog
        let mut expected = Buffer::with_lines(lines.clone());
        for y in 0..20 {
            for x in 0..50 {
                let ch = expected.get_mut(x, y);
                if y == 3 || y == 4 || y == 5 {
                    if x >= 2 && x <= 47 {
                        ch.set_fg(Color::Yellow);
                    }
                } else if y >= 7 && y <= 15 {
                    if x >= 2 && x <= 47 {
                        ch.set_fg(Color::White);
                    }
                } else {
                    ch.set_fg(Color::Reset);
                }
            }
        }
        test_case(&mut dialog, expected);
        // focus on '1 minute'
        dialog.next();
        dialog.next();
        dialog.next();
        let mut expected = Buffer::with_lines(lines.clone());
        for y in 0..20 {
            for x in 0..50 {
                let ch = expected.get_mut(x, y);
                if y == 9 {
                    if x >= 2 && x <= 47 {
                        ch.set_fg(Color::Yellow);
                    }
                } else if y == 3 || y == 4 || y == 5 {
                    if x >= 2 && x <= 47 {
                        ch.set_fg(Color::White);
                    }
                } else if y >= 7 && y <= 15 {
                    if x >= 2 && x <= 47 {
                        ch.set_fg(Color::White);
                    }
                } else {
                    ch.set_fg(Color::Reset);
                }
            }
        }
        test_case(&mut dialog, expected);
    }

    async fn simulate_input(
        dialog: &mut SearchConditionDialog<TestBackend>,
        input: &str,
        is_from: bool,
    ) {
        for ch in input.chars().into_iter() {
            dialog
                .handle_event(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE))
                .await;
        }
        if is_from {
            assert_eq!(input.to_string(), dialog.term_from.get_input());
        } else {
            assert_eq!(input.to_string(), dialog.term_to.get_input());
        }
    }

    #[tokio::test]
    async fn test_dialog_handle_event_basis() {
        let mut dialog: SearchConditionDialog<TestBackend> =
            SearchConditionDialog::new(SearchState::default());
        assert_eq!(0, dialog.focus);
        assert_eq!(
            SearchState::new("".to_string(), SearchMode::Tail),
            dialog.state
        );
        // Down
        assert!(
            dialog
                .handle_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE))
                .await
        );
        assert_eq!(1, dialog.focus);
        // Space
        assert!(
            dialog
                .handle_event(KeyEvent::new(KeyCode::Down, KeyModifiers::NONE))
                .await
        );
        assert!(
            dialog
                .handle_event(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE))
                .await
        );
        assert!(dialog.radios.one_minute.is_selected);
        assert_eq!(SearchMode::OneMinute, dialog.state.mode);
        // Up
        assert!(
            dialog
                .handle_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE))
                .await
        );
        assert_eq!(1, dialog.focus);
        // Tab
        dialog.focus = 6;
        assert_eq!(CustomInputMode::None, dialog.term_mode);
        assert!(
            dialog
                .handle_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE))
                .await
        );
        assert_eq!(CustomInputMode::From, dialog.term_mode);
    }

    #[tokio::test]
    async fn test_dialog_handle_event_input_custom_term() {
        let mut dialog: SearchConditionDialog<TestBackend> =
            SearchConditionDialog::new(SearchState::default());
        dialog.focus = 6;
        dialog.select();
        // input from
        assert!(
            dialog
                .handle_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE))
                .await
        );
        assert_eq!(CustomInputMode::From, dialog.term_mode);
        simulate_input(&mut dialog, "2021-01-01 00:00:01", true).await;
        // input to
        assert!(
            dialog
                .handle_event(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE))
                .await
        );
        assert_eq!(CustomInputMode::To, dialog.term_mode);
        simulate_input(&mut dialog, "2021-01-10 00:00:01", false).await;
        // get the timestamps
        let (from_timestamp, to_timestamp) = dialog.get_timestamps().unwrap();
        let from = Utc.timestamp(from_timestamp.unwrap() / 1000, 0);
        let to = Utc.timestamp(to_timestamp.unwrap() / 1000, 0);
        assert_eq!(chrono::Duration::days(9), to - from);
    }
}
