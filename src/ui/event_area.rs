use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use chrono::{DateTime, Local, TimeZone};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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
    loader::Loader,
    state::logevents_state::LogEventsState,
    ui::{search_condition_dialog::SearchConditionDialog, search_info::SearchInfo, Drawable},
};

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
        EventArea {
            log_group_name: log_group_name.to_string(),
            state,
            logevent_inst_tx,
            is_selected: false,
            loader: Loader::new(constant::LOADER.clone()),
            search_info: SearchInfo::default(),
            search_condition_dialog: SearchConditionDialog::default(),
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
            state = s.state.clone();
            s.events.items().iter().for_each(|item| {
                rows.push(
                    Row::new(vec![
                        // TODO: "v" when its fold flag is false
                        ">".to_string(),
                        if let Some(time) = item.timestamp {
                            let dt: DateTime<Local> = Local.timestamp(time / 1000, 0);
                            dt.to_string()
                        } else {
                            "".to_string()
                        },
                        if let Some(msg) = &item.message {
                            // TODO: insert newline when its fold flag is false
                            //let m = msg.clone();
                            // if m.len() > 10 {
                            //     m.insert_str(10, "\n");
                            // }
                            msg.clone()
                        } else {
                            "".to_string()
                        },
                    ])
                    .height(1),
                );
            });
            if s.is_fetching {
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
                .widths(&[
                    Constraint::Length(2),
                    Constraint::Percentage(20),
                    Constraint::Percentage(80),
                ])
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
            if let Selection::Search = self.selection {
                self.search_condition_dialog.handle_event(event).await;
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
                            self.search_info
                                .set_state(self.search_condition_dialog.get_state());
                            self.selection = Selection::Events;
                            // TODO: if search condition changed, reset events and fetch them
                        }
                        _ => {}
                    }
                } else {
                    if let KeyCode::Char(c) = event.code {
                        match c {
                            'j' => {
                                if let Ok(s) = state.as_mut() {
                                    s.next();
                                    if s.need_more_fetching() {
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
                            's' => {
                                if let KeyModifiers::CONTROL = event.modifiers {
                                    self.selection = Selection::Search;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            if need_more_fetching {
                let _ = self
                    .logevent_inst_tx
                    .send(LogEventEvent::FetchLogEvents(
                        self.log_group_name.clone(),
                        next_token,
                    ))
                    .await;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Local, TimeZone};
    use crossterm::event::{KeyCode, KeyModifiers};
    use tui::{backend::TestBackend, buffer::Buffer, style::Color};

    use super::*;
    use crate::logevents::LogEvents;
    use crate::test_helper::*;

    fn test_case(event_area: &mut EventArea<TestBackend>, color: Color, lines: Vec<&str>) {
        let mut terminal = get_test_terminal(100, 10);
        let lines = if !lines.is_empty() {
            lines
        } else {
            vec![
                "┌Events────────────────────────────────────────────────────────────────────────────────────────────┐",
                "│   Timestamp           Event                                                                      │",
                "│                                                                                                  │",
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
                if y == 0 || y == 9 {
                    ch.set_fg(color);
                } else if y == 1 {
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
        let lines = vec![
            "┌test-log-group────────────────────────────────────────────────────────────────────────────────────┐",
            "│   Timestamp           Event                                                                      │",
            &line1,
            &line2,
            &line3,
            "│                                                                                                  │",
            "│                                                                                                  │",
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
    }

    #[tokio::test]
    async fn test_handle_event() {
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
                let expected =
                    LogEventEvent::FetchLogEvents(log_group_name.clone(), Some(next_token.clone()));
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
        // input 'j', cursor up
        event_area.state.lock().unwrap().state.select(Some(1));
        assert!(
            !event_area
                .handle_event(KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE))
                .await
        );
        assert_eq!(Some(0), event_area.state.lock().unwrap().state.selected());
    }
}
