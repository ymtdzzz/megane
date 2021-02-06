use std::collections::BTreeMap;

use crossterm::event::KeyEvent;
use rusoto_logs::{CloudWatchLogsClient, FilteredLogEvent, LogGroup};
use rusoto_mock::{
    MockCredentialsProvider, MockRequestDispatcher, MockResponseReader, ReadMockResponse,
};
use tui::{backend::TestBackend, Terminal};

use crate::{constant::*, key_event_wrapper::KeyEventWrapper, ui::Drawable};

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

pub fn make_log_events(from: usize, to: usize, timestamp: u64) -> Vec<FilteredLogEvent> {
    let mut events = vec![];
    for i in from..=to {
        events.push(FilteredLogEvent {
            event_id: Some(i.to_string()),
            message: Some(format!("log_event_{}", i.to_string())),
            timestamp: Some((timestamp + ((i * 1000) as u64)) as i64),
            ..Default::default()
        });
    }
    events
}

pub fn get_events(from: usize, to: usize) -> Vec<FilteredLogEvent> {
    let mut events = vec![];
    for i in from..to {
        let mut event = FilteredLogEvent::default();
        event.event_id = Some(i.to_string());
        event.message = Some(i.to_string());
        event.timestamp = None;
        events.push(event);
    }
    events
}

pub fn get_test_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    let backend = TestBackend::new(width, height);
    Terminal::new(backend).unwrap()
}

pub fn key_maps_test_case<D>(component: &D, key_event: KeyEvent, expected: &str)
where
    D: Drawable<TestBackend>,
{
    let mut maps: BTreeMap<KeyEventWrapper, String> = BTreeMap::new();
    component.push_key_maps(&mut maps);
    assert_eq!(
        &expected.to_string(),
        maps.get(&KeyEventWrapper::new(key_event)).unwrap(),
    );
}
