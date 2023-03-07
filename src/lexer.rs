use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use crate::error::LexerError;

#[derive(Debug)]
pub enum JsonLexicalToken {
    Delim,
    Start,
    Object,
    Members,
    Pair,
    String,
    Value,
    Array,
    Elements,
    Chars,
    Char,

    // Terminal
    RightCurly,
    LeftCurly,
    Colon,
    Number,
    Bool,
    Quote,
    LeftSquareBracket,
    RightSquareBracket,
    Comma,
    Character,
}

pub struct JsonLexer {
    pub tokens: Vec<JsonLexicalToken>,
}

impl JsonLexer {
    pub fn new() -> JsonLexer {
        JsonLexer {
            tokens: vec![],
        }
    }
    pub fn consume(&mut self, c: char) -> Result<(), LexerError> {
        match c {
            'a'..='z' | 'A'..='Z' => self.tokens.push(JsonLexicalToken::Character),
            '{' => self.tokens.push(JsonLexicalToken::LeftCurly),
            '}' => self.tokens.push(JsonLexicalToken::RightCurly),
            '[' => self.tokens.push(JsonLexicalToken::LeftSquareBracket),
            ']' => self.tokens.push(JsonLexicalToken::RightSquareBracket),
            ':' => self.tokens.push(JsonLexicalToken::Colon),
            ',' => self.tokens.push(JsonLexicalToken::Comma),
            '\"' => self.tokens.push(JsonLexicalToken::Quote),
            '0'..='9' => self.tokens.push(JsonLexicalToken::Number),
            '\n' | ' ' | '\t' => {},
            _ => {
                return Err(LexerError::from(format!("Unrecognized char consumed by lexer '{}'", c)));
            }
        }
        return Ok(());
    }
}
