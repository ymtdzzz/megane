use tui::widgets::{ListItem, ListState};

use crate::{constant::MAX_LOG_GROUP_SELECTION, loggroups::*};

/// This struct is for managing log groups state.
pub struct LogGroupsState {
    pub log_groups: LogGroups,
    pub is_fetching: bool,
    pub selection: Vec<usize>,
}

impl LogGroupsState {
    pub fn new() -> Self {
        LogGroupsState {
            log_groups: LogGroups::new(vec![]),
            is_fetching: false,
            selection: vec![],
        }
    }

    pub fn get_list_items(&self) -> (Vec<ListItem>, ListState) {
        let items = self
            .log_groups
            .get_all_names()
            .iter()
            .enumerate()
            .map(|(i, v)| {
                if self.selection.contains(&i) {
                    ListItem::new(format!("[X]{}", v).to_owned())
                } else {
                    ListItem::new(format!("[ ]{}", v).to_owned())
                }
            })
            .collect::<Vec<ListItem>>();
        (items, self.log_groups.get_state())
    }

    pub fn select(&mut self, idx: usize) {
        if self.selection.contains(&idx) {
            let index = self.selection.iter().position(|v| *v == idx).unwrap();
            self.selection.remove(index);
        } else {
            if self.selection.len() < MAX_LOG_GROUP_SELECTION.clone() {
                self.selection.push(idx);
            }
        }
    }

    pub fn get_selected_log_group_names(&self) -> Vec<String> {
        let mut result = vec![];
        self.selection.iter().for_each(|item| {
            if let Some(i) = self.log_groups.get_item(item.clone()) {
                if let Some(name) = &i.log_group_name {
                    result.push(name.to_owned());
                }
            }
        });
        result
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
