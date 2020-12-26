use rusoto_logs::CloudWatchLogsClient;
use rusoto_mock::{
    MockCredentialsProvider, MockRequestDispatcher, MockResponseReader, ReadMockResponse,
};

#[ignore = "dead_code"]
pub fn get_mock_client(filename: &str) -> CloudWatchLogsClient {
    CloudWatchLogsClient::new_with(
        MockRequestDispatcher::default()
            .with_body(&MockResponseReader::read_response("mock_data", filename)),
        MockCredentialsProvider,
        Default::default(),
    )
}
