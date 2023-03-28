use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use crate::error::LexerError;

#[derive(Debug)]
pub enum JsonToken {
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
    Number(u32),
    Bool,
    Quote,
    LeftSquareBracket,
    RightSquareBracket,
    Comma,
    Character(char),
}

enum LexerState {
    InData,
    InString,
    InNumber,
}

pub struct JsonLexer<'a> {
    pub tokens: &'a mut Vec<JsonToken>,
    state: LexerState,
    buf: String,
}

impl<'a> JsonLexer<'a> {
    pub fn new(s: &'a mut Vec<JsonToken>) -> JsonLexer {
        JsonLexer {
            tokens: s,
            state: LexerState::InData,
            buf: String::new(),
        }
    }
    pub fn consume(&mut self, c: &u8) -> Result<(), LexerError> {
        let c = *c as char;
        let mut push = |t: JsonToken| {
            // println!("{:?}", t);
            self.tokens.push(t);
        };

        match self.state {
            LexerState::InData => {
                match c {
                    'a'..='z' | 'A'..='Z' => {self.tokens.push(JsonToken::Character(c)); ()}
                    '{' => push(JsonToken::LeftCurly),
                    '}' => push(JsonToken::RightCurly),
                    '[' => push(JsonToken::LeftSquareBracket),
                    ']' => push(JsonToken::RightSquareBracket),
                    ':' => push(JsonToken::Colon),
                    ',' => push(JsonToken::Comma),
                    '\"' => {
                        self.state = LexerState::InString;
                        push(JsonToken::Quote);
                    },
                    '0'..='9' => {
                        self.state = LexerState::InNumber;
                        self.buf.push(c);
                    },
                    '\n' | ' ' | '\t' => {},
                    _ => {
                        return Err(LexerError::from(format!("Unrecognized char consumed by lexer '{}'", c)));
                    }
                }
            }
            LexerState::InString => {
                match c {
                    '\"' => {
                        self.state = LexerState::InData;
                        push(JsonToken::Quote);
                    },
                    '\n' => {
                        return Err(LexerError::from("Cannot have newlines in strings".to_string()));
                    },
                    _ => push(JsonToken::Character(c)),
                }
            }
            LexerState::InNumber => {
                match c {
                    '0'..='9' => self.buf.push(c),
                    _ => {
                        self.state = LexerState::InData;
                        match self.buf.parse() {
                            Ok(num) => {
                                push(JsonToken::Number(num));
                                self.buf.clear();
                            },
                            Err(e) => return Err(LexerError::from(format!("Cannot parse string to u32 {:?}", e))),
                        }
                    },
                }
            }
        }
        return Ok(());
    }
}
