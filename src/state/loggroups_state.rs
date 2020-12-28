use tui::widgets::{ListItem, ListState};

use crate::loggroups::*;

/// This struct is for managing log groups state.
pub struct LogGroupsState {
    pub log_groups: LogGroups,
    pub is_fetching: bool,
}

impl LogGroupsState {
    pub fn new() -> Self {
        LogGroupsState {
            log_groups: LogGroups::new(vec![]),
            is_fetching: false,
        }
    }

    pub fn get_list_items(&self) -> (Vec<ListItem>, ListState) {
        let items = self
            .log_groups
            .get_all_names()
            .iter()
            .map(|i| ListItem::new(i.to_owned()))
            .collect::<Vec<ListItem>>();
        (items, self.log_groups.get_state())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let mut state = LogGroupsState::new();
        state.is_fetching = true;
        assert!(state.is_fetching);
    }
}
