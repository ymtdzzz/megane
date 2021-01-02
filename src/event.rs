pub enum LogGroupEvent {
    FetchLogGroups,
    Abort,
}

pub enum LogEventEvent {
    FetchLogEvents(String),
    Abort,
}

#[derive(Debug, PartialEq)]
pub enum Event<I> {
    Input(I),
    Tick,
}
