use std::str::Chars;


/// A parseable input.
pub trait Input<I> where I : Iterator<Item = char> {
    fn char_stream(self) -> I;
}

impl<'a, I> Input<I> for I where I : Iterator<Item = char> {
    fn char_stream(self) -> I {
        self
    }
}

impl<'a> Input<Chars<'a>> for &'a str {
    fn char_stream(self) -> Chars<'a> {
        self.chars()
    }
}

impl<'a> Input<Chars<'a>> for &'a String {
    fn char_stream(self) -> Chars<'a> {
        self.chars()
    }
}