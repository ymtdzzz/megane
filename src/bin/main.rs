use clap::{crate_authors, crate_description, crate_name, crate_version, App as ClapApp};
use rusoto_core::Region;
use rusoto_logs::CloudWatchLogsClient;

#[tokio::main]
async fn main() {
    let _clap = ClapApp::new(crate_name!())
        .author(crate_authors!())
        .version(crate_version!())
        .about(crate_description!())
        .get_matches();

    let _client = CloudWatchLogsClient::new(Region::ApNortheast1);
}
