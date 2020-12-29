#[derive(PartialEq, Debug)]
pub struct StatusBarState {
    pub message: String,
}

impl StatusBarState {
    pub fn new(message: String) -> Self {
        StatusBarState { message }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tset_new() {
        let result = StatusBarState::new(String::from("test message"));
        let expect = StatusBarState {
            message: String::from("test message"),
        };
        assert_eq!(expect, result);
    }
}
