use super::coord::InputCoord;


/// An internal parse error.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum ParseError {
    /// Production is completely absent, no characters have been read other than skipable characters
    Absent {
        from: InputCoord,
        to: InputCoord
    },

    /// Value is present but only partially or with invalid syntax, and some characters of it have already been read
    Error {
        from: InputCoord,
        to: InputCoord,
        msg: String
    }
}

impl ParseError {
    /// Converts an absent value into an error value with given message. If it is already an error value, it keeps it
    /// the same and doesn't change the message.
    pub fn to_error(self, msg: String) -> Self {
        if let Self::Absent { from, to } = self {
            Self::Error { from, to, msg }
        } else {
            self
        }
    }
}


/// A parse result, a [Result] of `T` or a [ParseError].
pub type ParseResult<T> = Result<T, ParseError>;