use std::{
    collections::BTreeMap,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use chrono::{DateTime, Local, TimeZone};
use clipboard::{ClipboardContext, ClipboardProvider};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use lazy_static::lazy_static;
use tokio::sync::mpsc;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table, TableState},
    Frame,
};

use crate::{
    constant,
    event::LogEventEvent,
    key_event_wrapper::KeyEventWrapper,
    loader::Loader,
    state::{
        logevents_state::LogEventsState,
        search_state::{SearchMode, SearchState},
    },
    ui::{search_condition_dialog::SearchConditionDialog, search_info::SearchInfo, Drawable},
};

lazy_static! {
    static ref TABLE_CONSTRAINT: [Constraint; 3] = [
        Constraint::Length(2),
        Constraint::Percentage(20),
        Constraint::Percentage(80),
    ];
}

#[derive(Debug, PartialEq)]
pub enum Selection {
    Events,
    Search,
}

pub struct EventArea<B>
where
    B: Backend,
{
    log_group_name: String,
    state: Arc<Mutex<LogEventsState>>,
    logevent_inst_tx: mpsc::Sender<LogEventEvent>,
    is_selected: bool,
    loader: Loader,
    search_info: SearchInfo<B>,
    search_condition_dialog: SearchConditionDialog<B>,
    selection: Selection,
    _phantom: PhantomData<B>,
}

impl<B> EventArea<B>
where
    B: Backend,
{
    pub fn new(
        log_group_name: &str,
        state: Arc<Mutex<LogEventsState>>,
        logevent_inst_tx: mpsc::Sender<LogEventEvent>,
    ) -> Self {
        let search_state = SearchState::new(String::default(), SearchMode::Tail);
        EventArea {
            log_group_name: log_group_name.to_string(),
            state,
            logevent_inst_tx,
            is_selected: false,
            loader: Loader::new(constant::LOADER.clone()),
            search_info: SearchInfo::new(search_state.clone()),
            search_condition_dialog: SearchConditionDialog::new(search_state),
            selection: Selection::Events,
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
        // dummy sender
        let (tx, _) = mpsc::channel(1);
        EventArea {
            log_group_name: String::from("Events"),
            state: Arc::new(Mutex::new(LogEventsState::default())),
            logevent_inst_tx: tx,
            is_selected: false,
            loader: Loader::new(constant::LOADER.clone()),
            search_info: SearchInfo::default(),
            search_condition_dialog: SearchConditionDialog::default(),
            selection: Selection::Events,
            _phantom: PhantomData,
        }
    }
}

#[async_trait]
impl<B> Drawable<B> for EventArea<B>
where
    B: Backend + Send,
{
    fn draw(&mut self, f: &mut Frame<'_, B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Percentage(100)].as_ref())
            .split(area);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(if self.is_selected {
                Style::default().fg(*constant::SELECTED_COLOR)
            } else {
                Style::default().fg(*constant::DESELECTED_COLOR)
            })
            .title(self.log_group_name.as_ref());
        let mut rows = vec![];
        let mut state = TableState::default();
        if let Ok(s) = self.state.try_lock() {
            // get event row width
            let table_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(TABLE_CONSTRAINT.as_ref())
                .split(chunks[1]);
            let width = table_chunks[2].width - 3;

            state = s.state.clone();
            let opened_idx_list = s.events.opened_idx();
            s.events.items().iter().enumerate().for_each(|(idx, item)| {
                let mut msg = if let Some(msg) = &item.message {
                    msg.clone()
                } else {
                    String::default()
                };
                let mut open = false;
                let (event_msg, row_height) = if opened_idx_list.contains(&idx) {
                    open = true;
                    insert_newline(&mut msg, width)
                } else {
                    (msg, 1)
                };
                rows.push(
                    Row::new(vec![
                        if open {
                            "v".to_string()
                        } else {
                            ">".to_string()
                        },
                        if let Some(time) = item.timestamp {
                            let dt: DateTime<Local> = Local.timestamp(time / 1000, 0);
                            dt.to_string()
                        } else {
                            "".to_string()
                        },
                        event_msg,
                    ])
                    .height(row_height),
                );
            });
            if self.search_condition_dialog.is_tail() {
                rows.push(Row::new(vec![
                    "".to_string(),
                    "Waiting for data...".to_string(),
                    "...".to_string(),
                ]));
            } else if s.is_fetching {
                rows.push(Row::new(vec![
                    // TODO: export function
                    self.loader.get_char().to_string(),
                    "".to_string(),
                    "".to_string(),
                ]));
            } else if s.next_token.is_some() {
                rows.push(Row::new(vec![
                    "".to_string(),
                    "More...".to_string(),
                    "...".to_string(),
                ]));
            }
        }
        let table = if let Selection::Events = self.selection {
            Table::new(rows)
                .block(block)
                .header(
                    Row::new(vec![" ", "Timestamp", "Event"])
                        .style(Style::default().fg(Color::White)),
                )
                .widths(TABLE_CONSTRAINT.as_ref())
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
                .style(Style::default())
                .column_spacing(1)
        } else {
            Table::new(rows).block(block)
        };

        self.search_info.draw(f, chunks[0]);
        f.render_stateful_widget(table, chunks[1], &mut state);
        if let Selection::Search = self.selection {
            self.search_condition_dialog.draw(f, chunks[1]);
        }
    }

    async fn handle_event(&mut self, event: KeyEvent) -> bool {
        if self.is_selected {
            let mut next_token = None;
            let mut need_more_fetching = false;
            let mut change_search_condition = false;
            if let Selection::Search = self.selection {
                if self.search_condition_dialog.handle_event(event).await {
                    return true;
                }
            }
            {
                let mut state = self.state.lock();
                if let Selection::Search = self.selection {
                    // search condition dialog event handling
                    match event.code {
                        KeyCode::Esc => {
                            self.selection = Selection::Events;
                        }
                        KeyCode::Enter => {
                            match self.search_condition_dialog.get_state() {
                                Ok(s) => {
                                    change_search_condition = !self.search_info.is_same_state(&s);
                                    self.search_info.set_state(s);
                                }
                                Err(e) => {
                                    println!("error: {:?}", e);
                                }
                            }
                            self.selection = Selection::Events;
                        }
                        _ => {}
                    }
                } else {
                    match event.code {
                        KeyCode::Enter => {
                            if let Ok(s) = state {
                                if let Some(idx) = s.state.selected() {
                                    let context: Result<
                                        ClipboardContext,
                                        Box<dyn std::error::Error>,
                                    > = ClipboardProvider::new();
                                    match context {
                                        Ok(mut ctx) => {
                                            if let Some(text) = s.events.get_message(idx) {
                                                ctx.set_contents(text.clone()).unwrap_or_else(|e| { log::warn!("Failed to write log event message to ClipboardContext: {}", e) });
                                                log::info!("Log event message has been written to Clipboard. \nlog event: \n{}", &text);
                                            }
                                        }
                                        Err(e) => {
                                            log::warn!("Failed to get ClipboardContext: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                        KeyCode::Tab => {
                            if let Ok(mut s) = state {
                                if let Some(idx) = s.state.selected() {
                                    s.events.toggle_select(idx);
                                }
                            }
                            return true;
                        }
                        KeyCode::Char(c) => match c {
                            'j' => {
                                if let Ok(s) = state.as_mut() {
                                    s.next();
                                    if !s.is_fetching && s.need_more_fetching() {
                                        next_token = s.next_token.clone();
                                        need_more_fetching = true;
                                    }
                                }
                            }
                            'J' => {
                                if let Ok(s) = state.as_mut() {
                                    s.next_by(*constant::LOGEVENT_STEP);
                                    if !s.is_fetching && s.need_more_fetching() {
                                        next_token = s.next_token.clone();
                                        need_more_fetching = true;
                                    }
                                }
                            }
                            'k' => {
                                if let Ok(s) = state.as_mut() {
                                    s.previous();
                                }
                            }
                            'K' => {
                                if let Ok(s) = state.as_mut() {
                                    s.previous_by(*constant::LOGEVENT_STEP);
                                }
                            }
                            's' => {
                                if let KeyModifiers::CONTROL = event.modifiers {
                                    self.selection = Selection::Search;
                                }
                            }
                            'g' => {
                                if let Ok(s) = state.as_mut() {
                                    s.cursor_first();
                                }
                            }
                            'G' => {
                                if let Ok(s) = state.as_mut() {
                                    s.cursor_last();
                                }
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
            let state = self.search_info.get_state();
            if change_search_condition {
                let _ = self
                    .logevent_inst_tx
                    .send(LogEventEvent::FetchLogEvents(
                        self.log_group_name.clone(),
                        next_token,
                        Some(state),
                        true,
                    ))
                    .await;
            } else if need_more_fetching {
                let _ = self
                    .logevent_inst_tx
                    .send(LogEventEvent::FetchLogEvents(
                        self.log_group_name.clone(),
                        next_token,
                        Some(state),
                        false,
                    ))
                    .await;
            }
        }
        false
    }

    fn push_key_maps<'a>(
        &self,
        maps: &'a mut BTreeMap<KeyEventWrapper, String>,
    ) -> &'a mut BTreeMap<KeyEventWrapper, String> {
        if let Selection::Search = self.selection {
            maps.insert(
                KeyEventWrapper::new(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)),
                "Cancel search dialog".to_string(),
            );
            maps.insert(
                KeyEventWrapper::new(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
                "Confirm search dialog".to_string(),
            );
            self.search_condition_dialog.push_key_maps(maps);
        } else {
            maps.insert(
                KeyEventWrapper::new(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE)),
                "Copy to clipboard".to_string(),
            );
            maps.insert(
                KeyEventWrapper::new(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE)),
                "Toggle log event open".to_string(),
            );
            maps.insert(
                KeyEventWrapper::new(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE)),
                "Next log event".to_string(),
            );
            maps.insert(
                KeyEventWrapper::new(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE)),
                "Prev log event".to_string(),
            );
            maps.insert(
                KeyEventWrapper::new(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL)),
                "Open search dialog".to_string(),
            );
        }
        maps
    }
}

// (row, height)
fn insert_newline(row: &mut String, width: u16) -> (String, u16) {
    let mut height: u16 = 1;
    let mut len = row.len() as u16;
    let mut current_pos = width;
    if len > width {
        while current_pos < len {
            row.insert(current_pos as usize, '\n');
            len = len.saturating_add(1);
            current_pos = current_pos.saturating_add(width + 1);
            height = height.saturating_add(1);
        }
    }
    (row.to_string(), height)
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Local, TimeZone};
    use crossterm::event::{KeyCode, KeyModifiers};
    use rusoto_logs::FilteredLogEvent;
    use tui::{backend::TestBackend, buffer::Buffer, style::Color};

    use super::*;
    use crate::logevents::LogEvents;
    use crate::state::search_state::SearchMode;
    use crate::test_helper::*;

    fn test_case(event_area: &mut EventArea<TestBackend>, color: Color, lines: Vec<&str>) {
        let mut terminal = get_test_terminal(100, 10);
        let lines = if !lines.is_empty() {
            lines
        } else {
            vec![
                "query: [], mode: [1 minute]                                                                         ",
                "┌Events────────────────────────────────────────────────────────────────────────────────────────────┐",
                "│   Timestamp           Event                                                                      │",
                "│                                                                                                  │",
                "│                                                                                                  │",
                "│                                                                                                  │",
                "│                                                                                                  │",
                "│                                                                                                  │",
                "│                                                                                                  │",
                "└──────────────────────────────────────────────────────────────────────────────────────────────────┘",
            ]
        };
        let mut expected = Buffer::with_lines(lines);
        for y in 0..10 {
            for x in 0..100 {
                let ch = expected.get_mut(x, y);
                if y == 1 || y == 9 {
                    ch.set_fg(color);
                } else if y == 2 {
                    if ch.symbol != "│" && ch.symbol != " " {
                        ch.set_fg(Color::White);
                    } else if ch.symbol == "│" {
                        ch.set_fg(color);
                    } else {
                        ch.set_fg(Color::White);
                    }
                } else if ch.symbol == "│" {
                    ch.set_fg(color);
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
        // default
        let mut event_area: EventArea<TestBackend> = EventArea::default();
        test_case(&mut event_area, Color::White, vec![]);
        event_area.set_select(true);
        test_case(&mut event_area, Color::Yellow, vec![]);
        // new
        let (tx, _) = mpsc::channel(1);
        let mut event_area: EventArea<TestBackend> = EventArea::new(
            "test-log-group",
            Arc::new(Mutex::new(LogEventsState::default())),
            tx,
        );
        let dt1: DateTime<Local> = Local.timestamp(1609426800, 0);
        let dt2: DateTime<Local> = Local.timestamp(1609426801, 0);
        let dt3: DateTime<Local> = Local.timestamp(1609426802, 0);
        let dt4: DateTime<Local> = Local.timestamp(1609426803, 0);
        let format = "%Y-%m-%d %H:%M:%S";
        let line1 = format!(
            "│>  {} log_event_0                                                                │",
            dt1.format(format).to_string()
        );
        let line2 = format!(
            "│>  {} log_event_1                                                                │",
            dt2.format(format).to_string()
        );
        let line3 = format!(
            "│>  {} log_event_2                                                                │",
            dt3.format(format).to_string()
        );
        let line4 = format!(
            "│v  {} 123456789012345678901234567890123456789012345678901234567890123456789012345│",
            dt4.format(format).to_string()
        );
        // default is Tail mode
        let lines = vec![
            "query: [], mode: [Tail]                                                                             ",
            "┌test-log-group────────────────────────────────────────────────────────────────────────────────────┐",
            "│   Timestamp           Event                                                                      │",
            &line1,
            &line2,
            &line3,
            "│   Waiting for data... ...                                                                        │",
            "│                                                                                                  │",
            "│                                                                                                  │",
            "└──────────────────────────────────────────────────────────────────────────────────────────────────┘",
        ];
        event_area
            .state
            .lock()
            .unwrap()
            .events
            .set_items(make_log_events(0, 2, 1609426800000));
        test_case(&mut event_area, Color::White, lines);
        // next token and opened event exist
        event_area
            .state
            .lock()
            .unwrap()
            .events
            .push_items(&mut vec![FilteredLogEvent {
                message: Some(String::from("123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890")),
                timestamp: Some(1609426803000),
                ..Default::default()
            }], false);
        event_area.state.lock().unwrap().events.toggle_select(3);
        event_area.state.lock().unwrap().next_token = Some(String::from("next_token"));
        let lines = vec![
            "query: [], mode: [Tail]                                                                             ",
            "┌test-log-group────────────────────────────────────────────────────────────────────────────────────┐",
            "│   Timestamp           Event                                                                      │",
            &line1,
            &line2,
            &line3,
            &line4,
            "│                       678901234567890                                                            │",
            "│   Waiting for data... ...                                                                        │",
            "└──────────────────────────────────────────────────────────────────────────────────────────────────┘",
        ];
        test_case(&mut event_area, Color::White, lines);
    }

    #[tokio::test]
    async fn test_handle_event_basis() {
        let log_group_name = String::from("test_log_gruop");
        let next_token = String::from("next_token");
        let (tx, mut rx) = mpsc::channel(1);
        let mut event_area: EventArea<TestBackend> = EventArea {
            log_group_name: log_group_name.clone(),
            logevent_inst_tx: tx,
            ..Default::default()
        };
        assert!(
            !event_area
                .handle_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE))
                .await
        );
        // input 'j', cursor down
        event_area.is_selected = true;
        event_area.state.lock().unwrap().events = LogEvents::new(make_log_events(0, 2, 0));
        event_area.state.lock().unwrap().state.select(Some(0));
        event_area.state.lock().unwrap().next_token = Some(next_token.clone());
        assert!(
            !event_area
                .handle_event(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE))
                .await
        );
        assert_eq!(Some(1), event_area.state.lock().unwrap().state.selected());
        assert!(
            !event_area
                .handle_event(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE))
                .await
        );
        assert_eq!(Some(2), event_area.state.lock().unwrap().state.selected());
        assert!(
            !event_area
                .handle_event(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE))
                .await
        );
        assert_eq!(Some(3), event_area.state.lock().unwrap().state.selected());
        // send an event to fetch more events
        let join = tokio::spawn(async move {
            if let Some(event) = rx.recv().await {
                let expected = LogEventEvent::FetchLogEvents(
                    log_group_name.clone(),
                    Some(next_token.clone()),
                    Some(SearchState::default()),
                    false,
                );
                assert_eq!(expected, event);
            }
        });
        assert!(
            !event_area
                .handle_event(KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE))
                .await
        );
        assert_eq!(Some(4), event_area.state.lock().unwrap().state.selected());
        let _ = join.await.unwrap();
        // input 'k', cursor up
        event_area.state.lock().unwrap().state.select(Some(1));
        assert!(
            !event_area
                .handle_event(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE))
                .await
        );
        assert_eq!(Some(0), event_area.state.lock().unwrap().state.selected());
        event_area.state.lock().unwrap().events = LogEvents::new(make_log_events(0, 6, 0));
        event_area.state.lock().unwrap().state.select(Some(0));
        assert!(
            !event_area
                .handle_event(KeyEvent::new(KeyCode::Char('J'), KeyModifiers::NONE))
                .await
        );
        assert_eq!(Some(5), event_area.state.lock().unwrap().state.selected());
        assert!(
            !event_area
                .handle_event(KeyEvent::new(KeyCode::Char('J'), KeyModifiers::NONE))
                .await
        );
        assert_eq!(Some(8), event_area.state.lock().unwrap().state.selected());
        assert!(
            !event_area
                .handle_event(KeyEvent::new(KeyCode::Char('K'), KeyModifiers::NONE))
                .await
        );
        assert_eq!(Some(3), event_area.state.lock().unwrap().state.selected());
        assert!(
            !event_area
                .handle_event(KeyEvent::new(KeyCode::Char('K'), KeyModifiers::NONE))
                .await
        );
        assert_eq!(Some(0), event_area.state.lock().unwrap().state.selected());
    }

    #[tokio::test]
    async fn test_handle_event_search_dialog() {
        let log_group_name = String::from("test_log_gruop");
        let (tx, _) = mpsc::channel(1);
        let original_search_info = SearchInfo::default();
        let changed_search_info = SearchInfo::new(SearchState::new(
            "query changed".to_string(),
            SearchMode::OneMinute,
        ));
        let mut event_area: EventArea<TestBackend> = EventArea {
            log_group_name: log_group_name.clone(),
            logevent_inst_tx: tx,
            is_selected: true,
            search_info: original_search_info.clone(),
            ..Default::default()
        };
        // show the search dialog up
        assert_eq!(event_area.selection, Selection::Events);
        assert!(
            !event_area
                .handle_event(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL))
                .await
        );
        assert_eq!(event_area.selection, Selection::Search);
        // Hit Enter, store the search query and close the dialog
        event_area.search_info = changed_search_info.clone();
        assert_ne!(
            original_search_info.get_state(),
            event_area.search_info.get_state()
        );
        assert!(
            !event_area
                .handle_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE))
                .await
        );
        assert_eq!(
            original_search_info.get_state(),
            event_area.search_info.get_state()
        );
        assert_eq!(event_area.selection, Selection::Events);
        // show up again
        assert!(
            !event_area
                .handle_event(KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL))
                .await
        );
        assert_eq!(event_area.selection, Selection::Search);
        // Hit Esc, just close the dialog (search query should not be changed)
        event_area.search_info = changed_search_info.clone();
        assert_ne!(
            original_search_info.get_state(),
            event_area.search_info.get_state()
        );
        assert!(
            !event_area
                .handle_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE))
                .await
        );
        assert_ne!(
            original_search_info.get_state(),
            event_area.search_info.get_state()
        );
    }

    #[test]
    fn test_insert_newline() {
        let (result_str, result_height) = insert_newline(
            &mut String::from("1234567890 abcdefghijklmn ABCDEFGHIJKLMN"),
            5,
        );
        let expected_str = String::from("12345\n67890\n abcd\nefghi\njklmn\n ABCD\nEFGHI\nJKLMN");
        assert_eq!(expected_str, result_str);
        assert_eq!(8, result_height);
    }
}
