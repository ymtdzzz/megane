use anyhow::{Error, Result};
use rusoto_logs::{
    CloudWatchLogs, CloudWatchLogsClient, DescribeLogGroupsRequest, FilterLogEventsRequest,
    LogGroup,
};

use super::loggroups::LogGroups;

/// rusoto wrapper
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
    pub async fn fetch_logs(&self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusoto_logs::LogGroup;
    use rusoto_mock::{
        MockCredentialsProvider, MockRequestDispatcher, MockResponseReader, ReadMockResponse,
    };

    fn get_mock_client(filename: &str) -> CloudWatchLogsClient {
        CloudWatchLogsClient::new_with(
            MockRequestDispatcher::default()
                .with_body(&MockResponseReader::read_response("mock_data", filename)),
            MockCredentialsProvider,
            Default::default(),
        )
    }

    fn make_log_groups(from: usize, to: usize) -> Vec<LogGroup> {
        let mut log_groups: Vec<LogGroup> = vec![];
        for i in from..=to {
            let mut group = LogGroup::default();
            group.arn = Some(i.to_string());
            group.log_group_name = Some(format!("log_group_{}", i.to_string()));
            log_groups.push(group);
        }
        log_groups
    }

    #[tokio::test]
    async fn test_fetch_log_groups() {
        let mock_client = get_mock_client("loggroups_01.json");
        let client = LogClient::new(mock_client);
        let result = client.fetch_log_groups().await.unwrap();
        let expect = make_log_groups(1, 3);
        assert_eq!(expect, result);
    }
}
