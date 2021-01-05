use rusoto_logs::FilteredLogEvent;


use super::constant::*;

#[derive(Debug)]
pub struct LogEvents {
    items: Vec<FilteredLogEvent>,
}

impl LogEvents {
    pub fn new(items: Vec<FilteredLogEvent>) -> Self {
        Self { items }
    }

    pub fn set_items(&mut self, items: Vec<FilteredLogEvent>) {
        self.items = items;
    }

    pub fn items(&self) -> &Vec<FilteredLogEvent> {
        &self.items
    }

    pub fn get_message(&self, idx: usize) -> Option<String> {
        if let Some(item) = self.items.get(idx) {
            item.message.clone()
        } else {
            None
        }
    }

    pub fn clear_items(&mut self) {
        self.items = vec![];
    }

    /// This method is used when pushing fetched items which possibly contains duplicate items.
    /// check:
    ///  - length
    ///  - first and last items are same
    pub fn is_same(&self, other: &Self) -> bool {
        let self_len = self.items.len();
        let other_len = other.items.len();
        if self_len != other_len {
            return false;
        }
        if let Some(first) = self.items.first() {
            if let Some(other_first) = other.items.first() {
                if first.event_id != other_first.event_id {
                    return false;
                }
            }
        }
        if let Some(last) = self
            .items
            .get(self_len.saturating_sub(if self.has_more_item() { 2 } else { 1 }))
        {
            if let Some(other_last) = other
                .items
                .get(other_len.saturating_sub(if other.has_more_item() { 2 } else { 1 }))
            {
                if last.event_id != other_last.event_id {
                    return false;
                }
            }
        }
        true
    }

    pub fn has_more_item(&self) -> bool {
        if let Some(last) = self.items.last() {
            last.event_id == Some(MORE_LOG_EVENT_ID.clone())
        } else {
            false
        }
    }

    pub fn push_items(&mut self, items: &mut Vec<FilteredLogEvent>, next_token: Option<&String>) {
        if self.has_more_item() {
            self.items.remove(self.items.len() - 1);
        }
        let mut idx: Option<usize> = None;
        // Skip the duplicate items.
        for (i, val) in items.iter().enumerate() {
            let mut found = false;
            for v in self.items.iter() {
                if val.event_id == v.event_id {
                    found = true;
                    break;
                }
            }
            if !found {
                idx = Some(i);
                break;
            }
        }
        if self.items.is_empty() {
            idx = Some(0);
        }
        if let Some(idx) = idx {
            let idx = idx;
            let mut items_to_push = items.split_off(idx);
            self.items.append(&mut items_to_push);
        }
        if next_token.is_some() {
            let more = FilteredLogEvent {
                event_id: Some(MORE_LOG_EVENT_ID.clone()),
                message: Some(String::from("")),
                timestamp: None,
                ..Default::default()
            };
            self.items.push(more);
        }
    }

    pub fn has_items(&self) -> bool {
        !self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::get_events;

    #[test]
    fn test_set_items() {
        let mut log_events = LogEvents::new(get_events(0, 4, false));
        let expected = LogEvents::new(get_events(3, 6, false));
        log_events.set_items(get_events(3, 6, false));
        for (i, val) in log_events.items.iter().enumerate() {
            assert_eq!(expected.items.get(i).unwrap(), val);
        }
    }

    #[test]
    fn test_get_message() {
        let log_events = LogEvents::new(get_events(0, 2, false));
        let msg1 = log_events.get_message(0);
        let msg2 = log_events.get_message(3);
        assert_eq!(Some(String::from("0")), msg1);
        assert_eq!(None, msg2);
    }

    #[test]
    fn test_clear_items() {
        let mut log_events = LogEvents::new(get_events(0, 2, false));
        log_events.clear_items();
        let expected: Vec<FilteredLogEvent> = vec![];
        assert_eq!(expected, log_events.items);
    }

    #[test]
    fn test_is_same() {
        // same length
        let log_events = LogEvents::new(get_events(0, 2, false));
        let same_log_events = LogEvents::new(get_events(0, 2, false));
        let diff_log_events = LogEvents::new(get_events(1, 3, false));
        let diff_log_events2 = LogEvents::new(get_events(0, 2, true));
        assert!(log_events.is_same(&same_log_events));
        assert!(!log_events.is_same(&diff_log_events));
        assert!(!log_events.is_same(&diff_log_events2));

        // different length
        let diff_log_events = LogEvents::new(get_events(0, 1, false));
        assert!(!log_events.is_same(&diff_log_events));

        // more items
        let log_events = LogEvents::new(get_events(0, 2, true));
        let mut diff_log_events = LogEvents::new(get_events(0, 3, true));
        diff_log_events.items.remove(1);
        assert!(!log_events.is_same(&diff_log_events));
    }

    #[test]
    fn test_has_more_item() {
        let log_events = LogEvents::new(get_events(0, 2, false));
        assert!(!log_events.has_more_item());
        let log_events = LogEvents::new(get_events(0, 2, true));
        assert!(log_events.has_more_item());
    }

    #[test]
    fn test_push_items() {
        let mut log_events = LogEvents::new(vec![]);
        let mut events = get_events(1, 2, false);
        log_events.push_items(&mut events, None);
        let mut events = get_events(2, 4, false);
        log_events.push_items(&mut events, Some(&String::from("Token")));
        let expected = LogEvents::new(get_events(1, 4, true));
        assert_eq!(expected.items.len(), log_events.items.len());
        for (i, val) in log_events.items.iter().enumerate() {
            assert_eq!(expected.items.get(i).unwrap(), val);
        }

        // has more item
        let mut log_events = LogEvents::new(get_events(0, 2, true));
        let mut events = get_events(0, 5, false);
        log_events.push_items(&mut events, None);
        let expected = LogEvents::new(get_events(0, 5, false));
        assert_eq!(expected.items.len(), log_events.items.len());
        for (i, val) in log_events.items.iter().enumerate() {
            assert_eq!(expected.items.get(i).unwrap(), val);
        }
    }
}
