use std::{
    str::Chars,
    iter::{
        Iterator,
        Cycle,
    },
};

pub struct Loader<'a> {
    loader: Cycle<Chars<'a>>,
}

impl<'a> Loader<'a> {
    pub fn new(chars: Chars<'a>) -> Self {
        Loader {
            loader: chars.cycle(),
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
        let mut loader = Loader::new("01234".chars());
        assert_eq!('0', loader.get_char());
        assert_eq!('1', loader.get_char());
        assert_eq!('2', loader.get_char());
        assert_eq!('3', loader.get_char());
        assert_eq!('4', loader.get_char());
        assert_eq!('0', loader.get_char());
    }
}

