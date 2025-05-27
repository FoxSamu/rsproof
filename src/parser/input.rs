use std::str::Chars;


/// A parseable input.
pub trait Input {
    type Iter : Iterator<Item = char>;

    fn char_stream(self) -> Self::Iter;
}


impl<'a> Input for &'a str {
    type Iter = Chars<'a>;

    fn char_stream(self) -> Self::Iter {
        self.chars()
    }
}

impl<'a> Input for &'a String {
    type Iter = Chars<'a>;

    fn char_stream(self) -> Self::Iter {
        self.chars()
    }
}

impl<'a> Input for String {
    type Iter = <Vec<char> as IntoIterator>::IntoIter;

    fn char_stream(self) -> Self::Iter {
        self.chars().collect::<Vec<_>>().into_iter()
    }
}