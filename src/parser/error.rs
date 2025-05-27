use std::fmt::Display;

use super::coord::{InputCoord, InputRange};

/// A parsing error.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Error {
    /// The error message
    pub msg: String,

    /// The start of the error
    pub from: InputCoord,

    /// The end of the error
    pub to: InputCoord
}

impl Error {
    /// Returns the [InputRange] spanning the whole error range.
    pub fn range(&self) -> InputRange {
        InputRange {
            from: self.from,
            to: self.to
        }
    }

    /// Prints the error as a nice message.
    pub fn display(&self) -> String {
        self.into()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}:{}] {}", self.from.line, self.from.col, self.msg)
    }
}

impl Into<String> for Error {
    fn into(self) -> String {
        format!("{}", self)
    }
}

impl Into<String> for &Error {
    fn into(self) -> String {
        format!("{}", self)
    }
}

impl std::error::Error for Error {
}