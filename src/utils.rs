use std::{collections::BTreeMap, str::FromStr};

use anyhow::Result;
use rusoto_core::{request::HttpClient, Region};
use rusoto_credential::ProfileProvider;
use rusoto_iam::{GetRoleRequest, Iam, IamClient};
use rusoto_logs::CloudWatchLogsClient;
use rusoto_sts::{StsAssumeRoleSessionCredentialsProvider, StsClient};
use tui::layout::Rect;

use crate::key_event_wrapper::KeyEventWrapper;

pub async fn get_aws_client(
    profile: Option<&str>,
    region: Option<&str>,
    role_name: Option<&str>,
    role_arn: Option<&str>,
) -> Result<CloudWatchLogsClient> {
    let region = if let Some(r) = region {
        Region::from_str(r)?
    } else {
        Region::default()
    };
    let mut assumed_provider = None;

    if let Some(arn) = role_arn {
        // If role_arn provided, assume the role
        let sts_client = StsClient::new_with(
            HttpClient::new()?,
            get_aws_provider(profile)?,
            region.clone(),
        );
        assumed_provider = Some(StsAssumeRoleSessionCredentialsProvider::new(
            sts_client,
            arn.to_string(),
            String::from("megane"),
            None,
            None,
            None,
            None,
        ));
    } else if let Some(name) = role_name {
        // If role_name provided, get the role's arn by its name and assume
        let iam_client = IamClient::new_with(
            HttpClient::new()?,
            get_aws_provider(profile)?,
            Region::UsEast1,
        );
        println!("Role name detected. Fetching the role data...");
        let arn = iam_client
            .get_role(GetRoleRequest {
                role_name: name.to_string(),
            })
            .await?
            .role
            .arn;
        println!("Completed to fetch the role's arn.");
        let sts_client = StsClient::new_with(
            HttpClient::new()?,
            get_aws_provider(profile)?,
            region.clone(),
        );
        assumed_provider = Some(StsAssumeRoleSessionCredentialsProvider::new(
            sts_client,
            arn,
            String::from("megane"),
            None,
            None,
            None,
            None,
        ));
    }
    if let Some(ap) = assumed_provider {
        Ok(CloudWatchLogsClient::new_with(
            HttpClient::new()?,
            ap,
            region,
        ))
    } else {
        Ok(CloudWatchLogsClient::new_with(
            HttpClient::new()?,
            get_aws_provider(profile)?,
            region,
        ))
    }
}

fn get_aws_provider(profile: Option<&str>) -> Result<ProfileProvider> {
    if let Some(p) = profile {
        Ok(ProfileProvider::with_default_credentials(p)?)
    } else {
        Ok(ProfileProvider::new()?)
    }
}

pub fn get_inner_area(area: &Rect) -> Rect {
    let mut area_cloned = *area;
    area_cloned.width = area.width - 2;
    area_cloned.height = area.height - 2;
    area_cloned.x = area.x + 1;
    area_cloned.y = area.y + 1;
    area_cloned
}

pub fn key_maps_stringify(maps: &BTreeMap<KeyEventWrapper, String>) -> String {
    let mut datas = vec![];
    for (k, v) in maps.iter() {
        let key = k.to_string();
        datas.push(format!("[{}]{}", key, v));
    }
    datas.join("/")
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::*;

    #[test]
    fn test_get_inner_area() {
        let rect = Rect {
            width: 100,
            height: 100,
            x: 0,
            y: 0,
            ..Default::default()
        };
        let expected = Rect {
            width: 98,
            height: 98,
            x: 1,
            y: 1,
            ..Default::default()
        };
        assert_eq!(expected, get_inner_area(&rect));
    }

    #[test]
    fn test_key_maps_string() {
        let mut input: BTreeMap<KeyEventWrapper, String> = BTreeMap::new();
        input.insert(
            KeyEventWrapper::new(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            String::from("test description 1"),
        );
        input.insert(
            KeyEventWrapper::new(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE)),
            String::from("test description 2"),
        );
        input.insert(
            KeyEventWrapper::new(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE)),
            String::from("test description 3"),
        );
        input.insert(
            KeyEventWrapper::new(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE)),
            String::from("test description 4"),
        );
        let result = key_maps_stringify(&input);
        let expected = String::from(
            "[BackSpace]test description 3/[C]test description 4/[C+Ctrl]test description 1/[SPC]test description 2",
        );
        assert_eq!(expected, result);
    }
}
