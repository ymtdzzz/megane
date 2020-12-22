use crate::loggroups::*;

/// This struct is for managing log groups state.
pub struct LogGroupsState {
    pub log_groups: LogGroups,
    pub next_token: Option<String>,
    pub is_fetching: bool,
}

impl LogGroupsState {
    pub fn new() -> Self {
        LogGroupsState {
            log_groups: LogGroups::new(vec![]),
            next_token: None,
            is_fetching: false,
        }
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
