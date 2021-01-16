#[derive(Debug, Clone)]
pub enum SearchMode {
    Tail,
    OneMinute,
    ThirtyMinutes,
    OneHour,
    TwelveHours,
    FromTo(u64, u64),
}

#[derive(Debug, Clone)]
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
