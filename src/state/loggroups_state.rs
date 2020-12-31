use rusoto_logs::LogGroup;
use tui::widgets::{ListItem, ListState};

use crate::{constant::MAX_LOG_GROUP_SELECTION, loggroups::*};

/// This struct is for managing log groups state.
pub struct LogGroupsState {
    pub log_groups: LogGroups,
    filtered_log_groups: LogGroups,
    pub is_fetching: bool,
    pub selection: Vec<usize>,
}

impl LogGroupsState {
    pub fn new() -> Self {
        LogGroupsState {
            log_groups: LogGroups::new(vec![]),
            filtered_log_groups: LogGroups::new(vec![]),
            is_fetching: false,
            selection: vec![],
        }
    }

    fn query_log_groups(&mut self, query: &str, exc: &Vec<String>) {
        self.filtered_log_groups.set_items(
            self.log_groups
                .items()
                .into_iter()
                .filter(|v| {
                    if let Some(gname) = &v.log_group_name {
                        query.is_empty()
                            || gname.contains(query)
                            || exc.contains(&gname.to_string())
                    } else {
                        false
                    }
                })
                .collect::<Vec<LogGroup>>(),
        );
    }

    pub fn get_list_items(
        &mut self,
        query: &str,
        exc: &Vec<String>,
    ) -> (Vec<ListItem<'_>>, ListState) {
        self.query_log_groups(query, exc);
        let items = self
            .filtered_log_groups
            .get_all_names()
            .iter()
            .enumerate()
            .map(|(i, v)| {
                if self.selection.contains(&i) {
                    ListItem::new(format!("[X]{}", v))
                } else {
                    ListItem::new(format!("[ ]{}", v))
                }
            })
            .collect::<Vec<ListItem<'_>>>();
        let mut state = self.log_groups.get_state();
        if let Some(idx) = state.selected() {
            if idx >= items.len() {
                let current_idx = items.len().saturating_sub(1);
                state.select(Some(current_idx));
                self.log_groups.state_select(current_idx);
            }
        }

        (items, state)
    }

    pub fn select(&mut self, idx: usize) {
        if self.selection.contains(&idx) {
            let index = self.selection.iter().position(|v| *v == idx).unwrap();
            self.selection.remove(index);
        } else if self.selection.len() < *MAX_LOG_GROUP_SELECTION {
            self.selection.push(idx);
        }
    }

    pub fn get_selected_log_group_names(&self) -> Vec<String> {
        let mut result = vec![];
        self.selection.iter().for_each(|item| {
            if let Some(i) = self.log_groups.get_item(*item) {
                if let Some(name) = &i.log_group_name {
                    result.push(name.to_owned());
                }
            }
        });
        result
    }
}

impl Default for LogGroupsState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use tui::widgets::{ListItem, ListState};

    use super::*;
    use crate::test_helper::make_log_groups;

    #[test]
    fn test_new() {
        let mut state = LogGroupsState::default();
        state.is_fetching = true;
        assert!(state.is_fetching);
    }

    #[test]
    fn test_get_list_items() {
        let mut state = LogGroupsState::default();
        state.log_groups = LogGroups::new(make_log_groups(0, 3));
        state.selection = vec![0, 2];
        let exp_item = vec![
            ListItem::new("[X]log_group_0"),
            ListItem::new("[ ]log_group_1"),
            ListItem::new("[X]log_group_2"),
            ListItem::new("[ ]log_group_3"),
        ];
        let exp_state = ListState::default();
        let (res_item, res_state) = state.get_list_items("", &vec![]);
        assert_eq!(exp_item, res_item);
        assert_eq!(exp_state.selected(), res_state.selected());
        let exp_item = vec![ListItem::new("[X]log_group_0")];
        let (res_item, res_state) = state.get_list_items("0", &vec![]);
        assert_eq!(exp_item, res_item);
        assert_eq!(exp_state.selected(), res_state.selected());
        let exp_item = vec![
            ListItem::new("[X]log_group_0"),
            ListItem::new("[ ]log_group_1"),
        ];
        let (res_item, res_state) = state.get_list_items("0", &vec!["log_group_1".to_string()]);
        assert_eq!(exp_item, res_item);
        assert_eq!(exp_state.selected(), res_state.selected());
    }

    #[test]
    fn test_select() {
        let mut state = LogGroupsState::default();
        state.selection = vec![0, 2];
        state.select(0);
        state.select(0);
        state.select(1);
        state.select(2);
        state.select(3);
        state.select(4);
        // don't add idx 5 because MAX_LOG_SELECTION is 4
        state.select(5);
        let expect = vec![0, 1, 3, 4];
        assert_eq!(expect, state.selection);
    }

    #[test]
    fn test_get_selected_log_group_names() {
        let mut state = LogGroupsState::default();
        state.log_groups = LogGroups::new(make_log_groups(0, 5));
        state.selection = vec![0, 2, 4];
        let result = state.get_selected_log_group_names();
        let mut expect = make_log_groups(0, 4);
        expect.remove(3);
        expect.remove(1);
        let expect = expect
            .iter_mut()
            .map(|v| v.log_group_name.as_ref().unwrap().to_string())
            .collect::<Vec<String>>();
        assert_eq!(expect, result);
    }
}
