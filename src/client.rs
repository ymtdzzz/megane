use anyhow::Result;
use rusoto_logs::{
    CloudWatchLogs, CloudWatchLogsClient, DescribeLogGroupsRequest, FilterLogEventsRequest,
    FilteredLogEvent, LogGroup,
};

/// rusoto wrapper
#[derive(Clone)]
pub struct LogClient {
    client: CloudWatchLogsClient,
}

impl LogClient {
    pub fn new(client: CloudWatchLogsClient) -> Self {
        LogClient { client }
    }

    /// Fetch all log groups
    pub async fn fetch_log_groups(&self) -> Result<Vec<LogGroup>> {
        let mut log_groups = vec![];
        let mut next_token = None;
        loop {
            let request = DescribeLogGroupsRequest {
                limit: Some(50),
                log_group_name_prefix: None,
                next_token: next_token.clone(),
            };
            let mut response = self.client.describe_log_groups(request).await?;
            if let Some(groups) = &mut response.log_groups {
                log_groups.append(groups);
            }
            next_token = response.next_token.clone();
            if next_token.is_none() {
                // All log groups fetched
                break;
            }
        }
        Ok(log_groups)
    }

    /// Fetch log events by query
    pub async fn fetch_logs(
        &self,
        log_group_name: &str,
        next_token: Option<String>,
    ) -> Result<(Vec<FilteredLogEvent>, Option<String>)> {
        let request = FilterLogEventsRequest {
            log_group_name: log_group_name.to_string(),
            limit: Some(10),
            next_token,
            ..Default::default()
        };
        let response = self.client.filter_log_events(request).await?;
        let events = if let Some(i) = response.events {
            i
        } else {
            vec![]
        };
        Ok((events, response.next_token))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::{get_mock_client, make_log_events, make_log_groups};

    #[tokio::test]
    async fn test_fetch_log_groups() {
        let mock_client = get_mock_client("loggroups_01.json");
        let client = LogClient::new(mock_client);
        let result = client.fetch_log_groups().await.unwrap();
        let expect = make_log_groups(1, 3);
        assert_eq!(expect, result);
    }

    #[tokio::test]
    async fn test_fetch_logs() {
        let mock_client = get_mock_client("logevents_01.json");
        let client = LogClient::new(mock_client);
        let (result, next_token) = client.fetch_logs("test-log-group", None).await.unwrap();
        let expect = make_log_events(1, 5, 1609426800000);
        assert!(next_token.is_some());
        assert_eq!(expect, result);
    }
}
