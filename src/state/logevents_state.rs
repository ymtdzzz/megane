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
                    if s <= self.events.items().len() {
                        self.state.select(Some(s.saturating_add(1)));
                    }
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

    pub fn need_more_fetching(&self) -> bool {
        if self.next_token.is_some() {
            if let Some(s) = self.state.selected() {
                return self.events.has_items() && s == self.events.items().len() + 1;
            }
        }
        false
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

    use crate::test_helper::*;

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

    #[test]
    fn test_next() {
        let mut state = LogEventsState::default();
        state.next(); // None
        assert!(state.state.selected().is_none());
        state.events = LogEvents::new(make_log_events(0, 1, 0));
        state.next(); // Some(0)
        assert_eq!(Some(0), state.state.selected());
        state.next(); // Some(1) means last item's idx
        assert_eq!(Some(1), state.state.selected());
        state.next(); // Some(2) means 'more ...' item
        assert_eq!(Some(2), state.state.selected());
        state.next(); // Some(3) means that should fetch more events
        assert_eq!(Some(3), state.state.selected());
        state.next();
        assert_eq!(Some(3), state.state.selected());

        // no events
        state.events = LogEvents::new(vec![]);
        state.state.select(Some(1));
        state.next();
        assert!(state.state.selected().is_none());
    }

    #[test]
    fn test_previous() {
        let mut state = LogEventsState::default();
        state.previous();
        assert!(state.state.selected().is_none());
        state.state.select(Some(1));
        state.previous();
        assert_eq!(Some(0), state.state.selected());
        state.previous();
        assert_eq!(Some(0), state.state.selected());
    }

    #[test]
    fn test_need_more_fetching() {
        let mut state = LogEventsState::default();
        assert!(!state.need_more_fetching());
        state.state.select(Some(2));
        assert!(!state.need_more_fetching());
        state.events = LogEvents::new(make_log_events(0, 2, 0));
        assert!(!state.need_more_fetching());
        state.state.select(Some(4));
        assert!(state.need_more_fetching());
    }
}
