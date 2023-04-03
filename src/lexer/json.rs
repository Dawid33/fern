use std::fmt::Debug;
use crate::grammar::Grammar;
use crate::lexer::error::LexerError;

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
#[allow(unused)]
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
    LeftSqrBracket,
    RightSqrBracket,
    Comma,
    Character(char),
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum LexerState {
    Start,
    InString,
    InNumber,
}

pub struct JsonLexer<'a> {
    pub tokens: &'a mut Vec<u8>,
    pub state: LexerState,
    buf: String,
    grammar: Grammar,
}

impl<'a> JsonLexer<'a> {
    pub fn new(grammar: Grammar, s: &'a mut Vec<u8>, start_state: LexerState) -> JsonLexer {
        JsonLexer {
            tokens: s,
            state: start_state,
            buf: String::new(),
            grammar,
        }
    }
    pub fn consume(&mut self, c: &u8) -> Result<(), LexerError> {
        loop {
            let mut should_reconsume = false;

            let c = *c as char;
            let mut push = |t: u8| {
                // println!("{:?}", t);
                self.tokens.push(t);
            };

            match self.state {
                LexerState::Start => match c {
                    'a'..='z' | 'A'..='Z' => {
                        self.tokens.push(0);
                    }
                    '{' => push(0),
                    '}' => push(0),
                    '[' => push(0),
                    ']' => push(0),
                    ':' => push(0),
                    ',' => push(0),
                    '\"' => {
                        self.state = LexerState::InString;
                        push(0);
                    }
                    '0'..='9' => {
                        self.state = LexerState::InNumber;
                        self.buf.push(c);
                    }
                    '\n' | ' ' | '\t' => {}
                    _ => {
                        return Err(LexerError::from(format!(
                            "Unrecognized char consumed by lexer '{}'",
                            c
                        )));
                    }
                },
                LexerState::InString => match c {
                    '\"' => {
                        self.state = LexerState::Start;
                        push(0);
                    }
                    '\n' => {
                        return Err(LexerError::from(
                            "Cannot have newlines in strings".to_string(),
                        ));
                    }
                    _ => push(0),
                },
                LexerState::InNumber => match c {
                    '0'..='9' => self.buf.push(c),
                    _ => {
                        self.state = LexerState::Start;
                        match self.buf.parse::<i64>() {
                            Ok(_) => {
                                push(0);
                                self.buf.clear();
                                should_reconsume = true;
                            }
                            Err(e) => {
                                return Err(LexerError::from(format!(
                                    "Cannot parse string to u32 {:?}",
                                    e
                                )))
                            }
                        }
                    }
                },
            }

            if !should_reconsume {
                break;
            }
        }
        return Ok(());
    }
}
