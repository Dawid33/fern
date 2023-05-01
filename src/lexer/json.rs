use crate::grammar::reader::TokenTypes;
use crate::grammar::{Grammar, Token};
use crate::lexer::error::LexerError;
use crate::lexer::LexerInterface;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum JsonLexerState {
    Start,
    InString,
    InNumber,
}

pub struct JsonTokens {
    pub lbrace: Token,
    pub rbrace: Token,
    pub lsquare: Token,
    pub rsquare: Token,
    pub comma: Token,
    pub colon: Token,
    pub bool: Token,
    pub quotes: Token,
    pub char: Token,
    pub number: Token,
}

impl JsonTokens {
    pub fn new(tokens_reverse: &HashMap<String, (Token, TokenTypes)>) -> JsonTokens {
        JsonTokens {
            lbrace: tokens_reverse.get("LBRACE").unwrap().0,
            rbrace: tokens_reverse.get("RBRACE").unwrap().0,
            lsquare: tokens_reverse.get("LSQUARE").unwrap().0,
            rsquare: tokens_reverse.get("RSQUARE").unwrap().0,
            comma: tokens_reverse.get("COMMA").unwrap().0,
            colon: tokens_reverse.get("COLON").unwrap().0,
            bool: tokens_reverse.get("BOOL").unwrap().0,
            quotes: tokens_reverse.get("QUOTES").unwrap().0,
            char: tokens_reverse.get("CHAR").unwrap().0,
            number: tokens_reverse.get("NUMBER").unwrap().0,
        }
    }
}

pub struct JsonLexer {
    pub tokens: Vec<Token>,
    pub data: HashMap<usize, String>,
    pub state: JsonLexerState,
    buf: String,
    grammar: Grammar,
    tok: JsonTokens,
}

impl LexerInterface<JsonLexerState> for JsonLexer {
    fn new(grammar: Grammar, start_state: JsonLexerState) -> JsonLexer {
        // Create a list of terminals that the lexer can output.
        // TODO: Figure out how to put this in the grammar file.
        JsonLexer {
            tokens: Vec::new(),
            state: start_state,
            buf: String::new(),
            data: HashMap::new(),
            tok: JsonTokens::new(&grammar.tokens_reverse),
            grammar,
        }
    }
    fn consume(&mut self, c: &u8) -> Result<(), LexerError> {
        loop {
            let mut should_reconsume = false;

            let c = *c as char;
            let mut push = |t: Token| {
                self.tokens.push(t);
            };

            match self.state {
                JsonLexerState::Start => match c {
                    'a'..='z' | 'A'..='Z' => {
                        push(self.tok.char);
                    }
                    '{' => push(self.tok.lbrace),
                    '}' => push(self.tok.rbrace),
                    '[' => push(self.tok.lsquare),
                    ']' => push(self.tok.rsquare),
                    ':' => push(self.tok.colon),
                    ',' => push(self.tok.comma),
                    '\"' => {
                        self.state = JsonLexerState::InString;
                        push(self.tok.quotes);
                    }
                    '0'..='9' => {
                        self.state = JsonLexerState::InNumber;
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
                JsonLexerState::InString => match c {
                    '\"' => {
                        self.state = JsonLexerState::Start;
                        push(self.tok.quotes);
                    }
                    '\n' => {
                        return Err(LexerError::from(
                            "Cannot have newlines in strings".to_string(),
                        ));
                    }
                    _ => push(self.tok.char),
                },
                JsonLexerState::InNumber => match c {
                    '0'..='9' => self.buf.push(c),
                    _ => {
                        self.state = JsonLexerState::Start;
                        push(self.tok.number);
                        self.data.insert(self.tokens.len(), self.buf.clone());
                        self.buf.clear();
                        should_reconsume = true;
                    }
                },
            }

            if !should_reconsume {
                break;
            }
        }
        return Ok(());
    }
    fn take(self) -> (JsonLexerState, Vec<Token>) {
        (self.state, self.tokens)
    }
}
