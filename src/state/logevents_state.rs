use crate::logevents::*;

use tui::widgets::TableState;

/// This struct is for managing log events state.
pub struct LogEventsState {
    pub events: LogEvents,
    pub next_token: Option<String>,
    pub is_fetching: bool,
    pub current_log_group: Option<String>,
    pub state: TableState,
}

impl LogEventsState {
    pub fn new() -> Self {
        LogEventsState {
            events: LogEvents::new(vec![]),
            next_token: None,
            is_fetching: false,
            current_log_group: None,
            state: TableState::default(),
        }
    }

    pub fn reset(&mut self) {
        self.events.clear_items();
        self.state = TableState::default();
        self.next_token = None;
    }

    pub fn next(&mut self) {
        match self.state.selected() {
            Some(s) => {
                if self.events.has_items() {
                    self.state.select(Some(s.saturating_add(1)));
                } else {
                    self.state.select(None);
                }
            }
            None => {
                if self.events.has_items() {
                    self.state.select(Some(0));
                } else {
                    self.state.select(None);
                }
            }
        }
    }

    pub fn previous(&mut self) {
        match self.state.selected() {
            Some(s) => {
                self.state.select(Some(s.saturating_sub(1)));
            }
            None => {
                self.state.select(None);
            }
        };
    }
}

impl Default for LogEventsState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusoto_logs::FilteredLogEvent;

    #[test]
    fn test_new() {
        let mut state = LogEventsState::default();
        state.is_fetching = true;
        assert!(state.is_fetching);
    }

    #[test]
    fn test_reset() {
        let mut state = LogEventsState::default();
        let mut events = vec![FilteredLogEvent::default()];
        state.events.push_items(&mut events, None);
        let expected = LogEventsState::default();
        assert!(!state.events.is_same(&expected.events));
        state.reset();
        assert!(state.events.is_same(&expected.events));
    }
}
