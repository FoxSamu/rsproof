use super::result::{ParseError, ParseResult};

/// A coordinate in the input.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct InputCoord {
    /// The character index, starting at 0 and increasing with each character read
    pub pos: usize,

    /// The line number, starting at 1 and increasing with each new line
    pub line: usize,

    /// The column number, starting at 1 and increasing with each character read, resetting to 1 when [line] increases
    pub col: usize,
}

impl InputCoord {
    /// Starting input coord.
    pub fn new() -> Self {
        Self {
            pos: 0,
            line: 1,
            col: 1
        }
    }

    /// Count a new line
    pub fn newline(&mut self) {
        self.pos += 1;
        self.line += 1;
        self.col = 1;
    }

    /// Count a non-newline
    pub fn advance(&mut self) {
        self.pos += 1;
        self.col += 1;
    }
}

/// A range of [InputCoord]s.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct InputRange {
    /// The starting coordinate
    pub from: InputCoord,

    /// The ending coordinate
    pub to: InputCoord
}

impl InputRange {
    /// Creates an absent [ParseResult] at this [InputRange].
    pub(super) fn absent<T>(&self) -> ParseResult<T> {
        return Err(ParseError::Absent {
            from: self.from,
            to: self.to
        });
    }

    /// Creates an error [ParseResult] at this [InputRange] and with given message.
    pub(super) fn error<T, S>(&self, msg: S) -> ParseResult<T> where S : Into<String> {
        return Err(ParseError::Error {
            from: self.from,
            to: self.to,
            msg: msg.into()
        });
    }
}