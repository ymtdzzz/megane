use std::fmt::{Display, Formatter, Result};

use chrono::{Duration, NaiveDateTime, Utc};

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
    pub fn get_timestamps(&self) -> (Option<i64>, Option<i64>) {
        match self {
            SearchMode::Tail => (None, None),
            SearchMode::OneMinute => {
                let from = Utc::now() - Duration::minutes(1);
                (Some(from.timestamp_millis()), None)
            }
            SearchMode::ThirtyMinutes => {
                let from = Utc::now() - Duration::minutes(30);
                (Some(from.timestamp_millis()), None)
            }
            SearchMode::OneHour => {
                let from = Utc::now() - Duration::hours(1);
                (Some(from.timestamp_millis()), None)
            }
            SearchMode::TwelveHours => {
                let from = Utc::now() - Duration::hours(12);
                (Some(from.timestamp_millis()), None)
            }
            SearchMode::FromTo(from, to) => (*from, *to),
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

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn test_from_duration(mode: SearchMode, expected_duration: Duration, check_minute: bool) {
        let (from, to) = mode.get_timestamps();
        let from_date = Utc.timestamp(from.unwrap() / 1000, 0);
        if check_minute {
            assert_eq!(
                (Utc::now() - from_date).num_seconds(),
                expected_duration.num_seconds(),
            );
        } else {
            assert_eq!(
                (Utc::now() - from_date).num_hours(),
                expected_duration.num_hours(),
            );
        }
        assert!(to.is_none());
    }

    fn test_mode_format(mode: SearchMode, expected: &str) {
        assert_eq!(expected, format!("{}", mode).as_str());
    }

    #[test]
    fn test_get_timestamps() {
        // tail
        let mode = SearchMode::Tail;
        assert_eq!((None, None), mode.get_timestamps());
        // presets
        test_from_duration(SearchMode::OneMinute, Duration::minutes(1), true);
        test_from_duration(SearchMode::ThirtyMinutes, Duration::minutes(30), true);
        test_from_duration(SearchMode::OneHour, Duration::hours(1), false);
        test_from_duration(SearchMode::TwelveHours, Duration::hours(12), false);
        // custom
        let duration = Duration::days(3);
        let to = Utc::now();
        let from = to - duration;
        let mode = SearchMode::FromTo(Some(from.timestamp_millis()), Some(to.timestamp_millis()));
        assert_eq!(
            (Some(from.timestamp_millis()), Some(to.timestamp_millis())),
            mode.get_timestamps()
        );
    }

    #[test]
    fn test_fmt() {
        test_mode_format(SearchMode::Tail, "Tail");
        test_mode_format(SearchMode::OneMinute, "1 minute");
        test_mode_format(SearchMode::ThirtyMinutes, "30 minutes");
        test_mode_format(SearchMode::OneHour, "1 hour");
        test_mode_format(SearchMode::TwelveHours, "12 hours");
        test_mode_format(SearchMode::FromTo(None, None), "~");
        let to_timstamp = Utc::now();
        let from_timestamp = to_timstamp - Duration::days(3);
        // truncate millis
        let to = Utc.timestamp(to_timstamp.timestamp(), 0).naive_utc();
        let from = Utc.timestamp(from_timestamp.timestamp(), 0).naive_utc();
        test_mode_format(
            SearchMode::FromTo(Some(from.timestamp_millis()), Some(to.timestamp_millis())),
            format!("{}~{}", from, to).as_str(),
        );
    }
}
