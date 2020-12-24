pub enum LogGroupEvent {
    FetchLogGroups,
    Abort,
}

#[derive(Debug)]
pub enum Event<I> {
    Input(I),
    Tick,
}
