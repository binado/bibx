use std::fmt;

/// What went wrong in a malformed item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorKind {
    ExpectedType,
    ExpectedDelimiter,
    ExpectedFieldName,
    ExpectedEquals,
    ExpectedValue,
    ExpectedSeparator,
    Unterminated,
}

/// A malformed item: what, where (byte offset of the failure) and a readable message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub message: String,
    pub offset: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (at byte {})", self.message, self.offset)
    }
}

impl std::error::Error for ParseError {}
