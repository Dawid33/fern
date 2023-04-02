use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct GrammarError {
    message: String,
}

impl Error for GrammarError {}

impl GrammarError {
    pub fn from(s: String) -> GrammarError {
        GrammarError { message: s }
    }
}

impl<'a> Display for GrammarError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Lexer Error: {}", self.message)
    }
}
