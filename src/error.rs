use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct LexerError {
    message: String,
}

impl Error for LexerError {}

impl LexerError {
    pub fn from(s: String) -> LexerError {
        LexerError { message: s }
    }
}

impl<'a> Display for LexerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Lexer Error: {}", self.message)
    }
}
