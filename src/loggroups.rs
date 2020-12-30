use rusoto_logs::LogGroup;
use tui::widgets::ListState;

use super::constant::*;

#[derive(Debug)]
pub struct LogGroups {
    items: Vec<LogGroup>,
    state: ListState,
}

impl LogGroups {
    pub fn new(items: Vec<LogGroup>) -> Self {
        LogGroups {
            items,
            state: ListState::default(),
        }
    }

    pub fn set_items(&mut self, items: Vec<LogGroup>) {
        self.items = items;
    }

    pub fn get_item(&self, idx: usize) -> Option<&LogGroup> {
        self.items.get(idx)
    }

    pub fn get_log_group_name(&self, idx: usize) -> Option<String> {
        if idx < self.items.len() {
            self.items[idx].log_group_name.clone()
        } else {
            None
        }
    }

    pub fn push_items(&mut self, mut items: &mut Vec<LogGroup>, has_next_token: bool) {
        if !self.items.is_empty() {
            self.items.remove(self.items.len() - 1);
        }
        self.items.append(&mut items);
        if has_next_token {
            let mut more = LogGroup::default();
            more.arn = Some(MORE_LOG_GROUP_ARN.clone());
            more.log_group_name = Some(MORE_LOG_GROUP_NAME.clone());
            self.items.push(more);
        }
    }

    pub fn has_more_items(&self) -> bool {
        if let Some(last) = self.items.last() {
            last.arn == Some(MORE_LOG_GROUP_ARN.clone())
        } else {
            false
        }
    }

    pub fn filter_items(&mut self, query: &str) {
        self.items = self
            .items
            .iter()
            .filter(|&item| {
                if let Some(log_group_name) = &item.log_group_name {
                    log_group_name.contains(query)
                } else {
                    false
                }
            })
            .cloned()
            .collect();
    }

    pub fn get_all_names(&self) -> Vec<&str> {
        self.items
            .iter()
            .filter(|i| i.log_group_name.is_some())
            .map(|i| i.log_group_name.as_ref().unwrap().as_str())
            .collect::<Vec<&str>>()
    }

    pub fn get_state(&self) -> ListState {
        self.state.clone()
    }

    pub fn get_current_idx(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn next(&mut self) {
        match self.state.selected() {
            Some(s) => {
                if !self.items.is_empty() {
                    self.state.select(Some(s.saturating_add(1)));
                } else {
                    self.state.select(None);
                }
            }
            None => {
                if !self.items.is_empty() {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::get_log_groups;

    #[test]
    fn test_setter() {
        let mut log_group_list = LogGroups::new(get_log_groups(0, 2, false));
        let expected = LogGroups::new(get_log_groups(98, 100, false));
        log_group_list.set_items(get_log_groups(98, 100, false));
        assert_eq!(expected.get_item(0), log_group_list.get_item(0));
        assert_eq!(expected.get_item(1), log_group_list.get_item(1));
        assert_eq!(expected.get_item(2), log_group_list.get_item(2));
    }

    #[test]
    fn test_getter() {
        let log_groups = LogGroups::new(get_log_groups(0, 4, false));
        let result = log_groups.get_log_group_name(2).unwrap();
        assert_eq!(String::from("log_group_2"), result);
        let result = log_groups.get_log_group_name(10);
        assert!(result.is_none());
    }

    #[test]
    fn test_has_more_item() {
        let log_groups = LogGroups::new(get_log_groups(0, 1, true));
        assert!(log_groups.has_more_items());
        let log_groups = LogGroups::new(get_log_groups(0, 1, false));
        assert_eq!(false, log_groups.has_more_items());
        let log_groups = LogGroups::new(vec![]);
        assert_eq!(false, log_groups.has_more_items());
    }

    #[test]
    fn test_filter_items() {
        let mut log_groups = LogGroups::new(get_log_groups(0, 3, true));
        log_groups.filter_items("log_group_");
        assert_eq!(4, log_groups.items.len());
        log_groups.items[0].log_group_name = None;
        log_groups.filter_items("log_group_");
        assert_eq!(3, log_groups.items.len());
    }

    #[test]
    fn test_push_items() {
        let mut log_groups = LogGroups::new(get_log_groups(0, 2, true));
        let mut groups = get_log_groups(3, 5, false);
        log_groups.push_items(&mut groups, true);
        let expected = LogGroups::new(get_log_groups(0, 5, true));
        assert_eq!(expected.items, log_groups.items);

        let mut log_groups = LogGroups::new(get_log_groups(0, 2, true));
        let mut groups = get_log_groups(3, 5, false);
        log_groups.push_items(&mut groups, false);
        let expected = LogGroups::new(get_log_groups(0, 5, false));
        assert_eq!(expected.items, log_groups.items);
    }
}
