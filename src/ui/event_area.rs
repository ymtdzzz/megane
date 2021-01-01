use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use chrono::{DateTime, Local, TimeZone};
use crossterm::event::KeyEvent;
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Row, Table, TableState},
    Frame,
};

use crate::{constant, state::logevents_state::LogEventsState, ui::Drawable};

pub struct EventArea<B>
where
    B: Backend,
{
    log_group_name: String,
    state: Arc<Mutex<LogEventsState>>,
    is_selected: bool,
    _phantom: PhantomData<B>,
}

impl<B> EventArea<B>
where
    B: Backend,
{
    pub fn new(log_group_name: &str, state: Arc<Mutex<LogEventsState>>) -> Self {
        EventArea {
            log_group_name: log_group_name.to_string(),
            state,
            is_selected: false,
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
        EventArea {
            log_group_name: String::from("Events"),
            state: Arc::new(Mutex::new(LogEventsState::default())),
            is_selected: false,
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
        if let Ok(s) = self.state.try_lock() {
            s.events.items().iter().for_each(|item| {
                rows.push(Row::Data(
                    vec![
                        if let Some(time) = item.timestamp {
                            let dt: DateTime<Local> = Local.timestamp(time, 0);
                            dt.to_string()
                        } else {
                            "".to_string()
                        },
                        if let Some(msg) = &item.message {
                            msg.to_string()
                        } else {
                            "".to_string()
                        },
                    ]
                    .into_iter(),
                ));
            });
        }
        let table = Table::new(constant::LOG_EVENTS_HEADER.iter(), rows.into_iter())
            .block(block)
            .header_style(Style::default().fg(Color::White))
            .widths(&[Constraint::Percentage(20), Constraint::Percentage(80)])
            .style(Style::default())
            .column_spacing(1);

        f.render_stateful_widget(table, area, &mut TableState::default());
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
    use crate::test_helper::{get_test_terminal, make_log_events};

    fn test_case(event_area: &mut EventArea<TestBackend>, color: Color, lines: Vec<&str>) {
        let mut terminal = get_test_terminal(100, 10);
        let lines = if !lines.is_empty() {
            lines
        } else {
            vec![
                "┌Events────────────────────────────────────────────────────────────────────────────────────────────┐",
                "│timestamp            event                                                                        │",
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
                        ch.set_fg(Color::Reset);
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
        let mut event_area: EventArea<TestBackend> = EventArea::new(
            "test-log-group",
            Arc::new(Mutex::new(LogEventsState::default())),
        );
        let lines = vec![
            "┌test-log-group────────────────────────────────────────────────────────────────────────────────────┐",
            "│timestamp            event                                                                        │",
            "│                                                                                                  │",
            "│2021-01-01 00:00:00  log_event_0                                                                  │",
            "│2021-01-01 00:00:01  log_event_1                                                                  │",
            "│2021-01-01 00:00:02  log_event_2                                                                  │",
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
            .set_items(make_log_events(0, 2, 1609426800));
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
