use std::{
    iter::{Cycle, Iterator},
    vec::IntoIter,
};

pub struct Loader {
    loader: Cycle<IntoIter<char>>,
}

impl Loader {
    pub fn new(s: String) -> Self {
        Loader {
            loader: s.chars().collect::<Vec<_>>().into_iter().cycle(),
        }
    }

    pub fn get_char(&mut self) -> char {
        self.loader.next().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_char() {
        let mut loader = Loader::new("01234".to_string());
        assert_eq!('0', loader.get_char());
        assert_eq!('1', loader.get_char());
        assert_eq!('2', loader.get_char());
        assert_eq!('3', loader.get_char());
        assert_eq!('4', loader.get_char());
        assert_eq!('0', loader.get_char());
    }
}
