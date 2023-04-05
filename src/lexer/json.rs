use std::collections::HashMap;
use std::fmt::Debug;
use log::trace;
use crate::grammar::Grammar;
use crate::lexer::error::LexerError;

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
    LBRACE: u8,
    RBRACE: u8,
    LSQUARE: u8,
    RSQUARE: u8,
    COMMA: u8,
    COLON: u8,
    BOOL: u8,
    QUOTES: u8,
    CHAR: u8,
    NUMBER: u8,
}


impl<'a> JsonLexer<'a> {
    pub fn new(grammar: Grammar, s: &'a mut Vec<u8>, start_state: LexerState) -> JsonLexer {
        // Create a list of terminals that the lexer can output.
        // TODO: Figure out how to put this in the grammar file.
        JsonLexer {
            tokens: s,
            state: start_state,
            buf: String::new(),
            LBRACE: grammar.tokens_reverse.get("LBRACE").unwrap().0,
            RBRACE: grammar.tokens_reverse.get("RBRACE").unwrap().0,
            LSQUARE: grammar.tokens_reverse.get("LSQUARE").unwrap().0,
            RSQUARE: grammar.tokens_reverse.get("RSQUARE").unwrap().0,
            COMMA: grammar.tokens_reverse.get("COMMA").unwrap().0,
            COLON: grammar.tokens_reverse.get("COLON").unwrap().0,
            BOOL: grammar.tokens_reverse.get("BOOL").unwrap().0,
            QUOTES: grammar.tokens_reverse.get("QUOTES").unwrap().0,
            CHAR: grammar.tokens_reverse.get("CHAR").unwrap().0,
            NUMBER: grammar.tokens_reverse.get("NUMBER").unwrap().0,
            grammar,
        }
    }
    pub fn consume(&mut self, c: &u8) -> Result<(), LexerError> {
        loop {
            let mut should_reconsume = false;

            let c = *c as char;
            let mut push = |t: u8| {
                trace!("{:?}", t);
                self.tokens.push(t);
            };

            match self.state {
                LexerState::Start => match c {
                    'a'..='z' | 'A'..='Z' => {
                        push(self.CHAR);
                    }
                    '{' => push(self.LBRACE),
                    '}' => push(self.RBRACE),
                    '[' => push(self.LSQUARE),
                    ']' => push(self.RSQUARE),
                    ':' => push(self.COLON),
                    ',' => push(self.COMMA),
                    '\"' => {
                        self.state = LexerState::InString;
                        push(self.QUOTES);
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
                        push(self.QUOTES);
                    }
                    '\n' => {
                        return Err(LexerError::from(
                            "Cannot have newlines in strings".to_string(),
                        ));
                    }
                    _ => push(self.CHAR),
                },
                LexerState::InNumber => match c {
                    '0'..='9' => self.buf.push(c),
                    _ => {
                        self.state = LexerState::Start;
                        match self.buf.parse::<i64>() {
                            Ok(_) => {
                                push(self.NUMBER);
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
