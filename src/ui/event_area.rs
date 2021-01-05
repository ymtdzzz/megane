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
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Row, Table, TableState},
    Frame,
};

use crate::{
    constant, event::LogEventEvent, loader::Loader, state::logevents_state::LogEventsState,
    ui::Drawable,
};

pub struct EventArea<B>
where
    B: Backend,
{
    log_group_name: String,
    state: Arc<Mutex<LogEventsState>>,
    logevent_inst_tx: mpsc::Sender<LogEventEvent>,
    is_selected: bool,
    loader: Loader,
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
                            let m = msg.clone();
                            // if m.len() > 10 {
                            //     m.insert_str(10, "\n");
                            // }
                            m
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
        let table = Table::new(rows)
            .header(
                Row::new(vec![" ", "Timestamp", "Event"]).style(Style::default().fg(Color::White)),
            )
            .block(block)
            .widths(&[
                Constraint::Length(2),
                Constraint::Percentage(20),
                Constraint::Percentage(80),
            ])
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .style(Style::default())
            .column_spacing(1);

        f.render_stateful_widget(table, area, &mut state);
    }

    async fn handle_event(&mut self, event: KeyEvent) -> bool {
        if self.is_selected {
            let mut next_token = None;
            let mut need_more_fetching = false;
            {
                let mut state = self.state.lock();
                match event.code {
                    KeyCode::Char(c) => match c {
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
                        _ => {}
                    },
                    _ => {}
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
    use crate::test_helper::{get_test_terminal, make_log_events};

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
        let mut event_area: EventArea<TestBackend> = EventArea::default();
        assert!(
            !event_area
                .handle_event(KeyEvent::new(KeyCode::Up, KeyModifiers::NONE))
                .await
        );
    }
}
