use anyhow::Result;
use rusoto_logs::{
    CloudWatchLogs, CloudWatchLogsClient, DescribeLogGroupsRequest, FilterLogEventsRequest,
};

/// rusoto wrapper
pub struct LogClient {
    client: CloudWatchLogsClient,
}

impl LogClient {
    pub fn new(client: CloudWatchLogsClient) -> Self {
        LogClient { client }
    }

    pub async fn fetch_log_groups(&self) {}

    pub async fn fetch_logs(&self) {}
}
