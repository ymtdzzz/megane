use std::fmt::{Display, Formatter, Result};

use chrono::{DateTime, Duration, NaiveDateTime, TimeZone, Utc};

#[derive(Debug, Clone, PartialEq)]
pub enum SearchMode {
    Tail,
    OneMinute,
    ThirtyMinutes,
    OneHour,
    TwelveHours,
    FromTo(Option<i64>, Option<i64>),
}

impl SearchMode {
    pub fn get_timestamps(&self) -> anyhow::Result<(Option<i64>, Option<i64>)> {
        match self {
            SearchMode::Tail => Ok((None, None)),
            SearchMode::OneMinute => {
                let from = Utc::now() - Duration::minutes(1);
                Ok((Some(from.timestamp_millis()), None))
            }
            SearchMode::ThirtyMinutes => {
                let from = Utc::now() - Duration::minutes(30);
                Ok((Some(from.timestamp_millis()), None))
            }
            SearchMode::OneHour => {
                let from = Utc::now() - Duration::hours(1);
                Ok((Some(from.timestamp_millis()), None))
            }
            SearchMode::TwelveHours => {
                let from = Utc::now() - Duration::hours(12);
                Ok((Some(from.timestamp_millis()), None))
            }
            SearchMode::FromTo(from, to) => Ok((*from, *to)),
        }
    }
}

impl Display for SearchMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            SearchMode::Tail => {
                write!(f, "Tail")
            }
            SearchMode::OneMinute => {
                write!(f, "1 minute")
            }
            SearchMode::ThirtyMinutes => {
                write!(f, "30 minutes")
            }
            SearchMode::OneHour => {
                write!(f, "1 hour")
            }
            SearchMode::TwelveHours => {
                write!(f, "12 hours")
            }
            SearchMode::FromTo(from, to) => {
                let from = if let Some(time) = from {
                    NaiveDateTime::from_timestamp(time / 1000, 0).to_string()
                } else {
                    String::default()
                };
                let to = if let Some(time) = to {
                    NaiveDateTime::from_timestamp(time / 1000, 0).to_string()
                } else {
                    String::default()
                };
                write!(f, "{}~{}", from, to)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SearchState {
    pub query: String,
    pub mode: SearchMode,
}

impl SearchState {
    pub fn new(query: String, mode: SearchMode) -> Self {
        SearchState { query, mode }
    }
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new("".to_string(), SearchMode::Tail)
    }
}
