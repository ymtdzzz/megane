use rusoto_logs::{CloudWatchLogsClient, FilteredLogEvent, LogGroup};
use rusoto_mock::{
    MockCredentialsProvider, MockRequestDispatcher, MockResponseReader, ReadMockResponse,
};

use super::*;
use crate::constant::*;

#[cfg(test)]
pub fn get_mock_client(filename: &str) -> CloudWatchLogsClient {
    CloudWatchLogsClient::new_with(
        MockRequestDispatcher::default()
            .with_body(&MockResponseReader::read_response("mock_data", filename)),
        MockCredentialsProvider,
        Default::default(),
    )
}

pub fn make_log_groups(from: usize, to: usize) -> Vec<LogGroup> {
    let mut log_groups: Vec<LogGroup> = vec![];
    for i in from..=to {
        let mut group = LogGroup::default();
        group.arn = Some(i.to_string());
        group.log_group_name = Some(format!("log_group_{}", i.to_string()));
        log_groups.push(group);
    }
    log_groups
}

pub fn get_log_groups(from: usize, to: usize, has_more: bool) -> Vec<LogGroup> {
    let mut groups = vec![];
    for i in from..=to {
        let mut group = LogGroup::default();
        group.arn = Some(i.to_string());
        group.log_group_name = Some(format!("log_group_{}", i.to_string()));
        groups.push(group);
    }
    if has_more {
        let mut group = LogGroup::default();
        group.arn = Some(MORE_LOG_GROUP_ARN.clone());
        group.log_group_name = Some(MORE_LOG_GROUP_NAME.clone());
        groups.push(group);
    }
    groups
}

pub fn get_events(from: usize, to: usize, has_more: bool) -> Vec<FilteredLogEvent> {
    let mut events = vec![];
    for i in from..to {
        let mut event = FilteredLogEvent::default();
        event.event_id = Some(i.to_string());
        event.message = Some(i.to_string());
        event.timestamp = None;
        events.push(event);
    }
    if has_more {
        let mut event = FilteredLogEvent::default();
        event.event_id = Some(MORE_LOG_EVENT_ID.clone());
        event.message = Some(String::from(""));
        event.timestamp = None;
        events.push(event);
    }
    events
}
