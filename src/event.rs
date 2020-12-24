pub enum LogGroupEvent {
    FetchLogGroups,
    Abort,
}

#[derive(Debug, PartialEq)]
pub enum Event<I> {
    Input(I),
    Tick,
}
