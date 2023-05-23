use crate::grammar::reader::TokenTypes;
use crate::grammar::{OpGrammar, Token};
use crate::lexer::error::LexerError;
use crate::lexer::LexerInterface;
use log::trace;
use std::collections::HashMap;
use std::fmt::Debug;
use crate::lexer::fern::FernData::NoData;
use crate::lexer::fern::FernLexerState::{InFunctionDef, Start};
use crate::lexer::fern::InLiteral::{Name, Number};
use crate::lexer::lua::LuaLexer;

pub struct FernTokens {
    pub endfile: Token,
    pub return_t: Token,
    pub semi: Token,
    pub colon: Token,
    pub colon2: Token,
    pub dot: Token,
    pub dot3: Token,
    pub comma: Token,
    pub lbrack: Token,
    pub rbrack: Token,
    pub lbrace: Token,
    pub rbrace: Token,
    pub lparen: Token,
    pub rparen: Token,
    pub break_t: Token,
    pub goto: Token,
    pub do_t: Token,
    pub end: Token,
    pub while_t: Token,
    pub repeat: Token,
    pub until: Token,
    pub if_t: Token,
    pub then: Token,
    pub elseif: Token,
    pub else_t: Token,
    pub for_t: Token,
    pub in_t: Token,
    pub nil: Token,
    pub false_t: Token,
    pub true_t: Token,
    pub number: Token,
    pub string: Token,
    pub name: Token,
    pub plus: Token,
    pub minus: Token,
    pub asterisk: Token,
    pub divide: Token,
    pub caret: Token,
    pub percent: Token,
    pub dot2: Token,
    pub lt: Token,
    pub gt: Token,
    pub lteq: Token,
    pub gteq: Token,
    pub eq2: Token,
    pub neq: Token,
    pub and: Token,
    pub or: Token,
    pub not: Token,
    pub uminus: Token,
    pub sharp: Token,
    pub lparenfunc: Token,
    pub rparenfunc: Token,
    pub semifield: Token,
    pub eq: Token,
    pub let_t: Token,
    pub fn_t: Token,
    pub questionmark: Token,
    pub struct_t: Token,
}

impl FernTokens {
    pub fn new(tokens_reverse: &HashMap<String, (Token, TokenTypes)>) -> FernTokens {
        FernTokens {
            endfile: tokens_reverse.get("ENDFILE").unwrap().0,
            return_t: tokens_reverse.get("RETURN").unwrap().0,
            semi: tokens_reverse.get("SEMI").unwrap().0,
            colon: tokens_reverse.get("COLON").unwrap().0,
            colon2: tokens_reverse.get("COLON2").unwrap().0,
            dot: tokens_reverse.get("DOT").unwrap().0,
            dot3: tokens_reverse.get("DOT3").unwrap().0,
            comma: tokens_reverse.get("COMMA").unwrap().0,
            lbrack: tokens_reverse.get("LBRACK").unwrap().0,
            rbrack: tokens_reverse.get("RBRACK").unwrap().0,
            lbrace: tokens_reverse.get("LBRACE").unwrap().0,
            rbrace: tokens_reverse.get("RBRACE").unwrap().0,
            lparen: tokens_reverse.get("LPAREN").unwrap().0,
            rparen: tokens_reverse.get("RPAREN").unwrap().0,
            eq: tokens_reverse.get("EQ").unwrap().0,
            break_t: tokens_reverse.get("BREAK").unwrap().0,
            goto: tokens_reverse.get("GOTO").unwrap().0,
            do_t: tokens_reverse.get("DO").unwrap().0,
            end: tokens_reverse.get("END").unwrap().0,
            while_t: tokens_reverse.get("WHILE").unwrap().0,
            repeat: tokens_reverse.get("REPEAT").unwrap().0,
            until: tokens_reverse.get("UNTIL").unwrap().0,
            if_t: tokens_reverse.get("IF").unwrap().0,
            then: tokens_reverse.get("THEN").unwrap().0,
            elseif: tokens_reverse.get("ELSEIF").unwrap().0,
            else_t: tokens_reverse.get("ELSE").unwrap().0,
            for_t: tokens_reverse.get("FOR").unwrap().0,
            in_t: tokens_reverse.get("IN").unwrap().0,
            fn_t: tokens_reverse.get("FUNCTION").unwrap().0,
            let_t: tokens_reverse.get("LET").unwrap().0,
            nil: tokens_reverse.get("NIL").unwrap().0,
            false_t: tokens_reverse.get("FALSE").unwrap().0,
            true_t: tokens_reverse.get("TRUE").unwrap().0,
            number: tokens_reverse.get("NUMBER").unwrap().0,
            string: tokens_reverse.get("STRING").unwrap().0,
            name: tokens_reverse.get("NAME").unwrap().0,
            plus: tokens_reverse.get("PLUS").unwrap().0,
            minus: tokens_reverse.get("MINUS").unwrap().0,
            asterisk: tokens_reverse.get("ASTERISK").unwrap().0,
            divide: tokens_reverse.get("DIVIDE").unwrap().0,
            caret: tokens_reverse.get("CARET").unwrap().0,
            percent: tokens_reverse.get("PERCENT").unwrap().0,
            dot2: tokens_reverse.get("DOT2").unwrap().0,
            lt: tokens_reverse.get("LT").unwrap().0,
            gt: tokens_reverse.get("GT").unwrap().0,
            lteq: tokens_reverse.get("LTEQ").unwrap().0,
            gteq: tokens_reverse.get("GTEQ").unwrap().0,
            eq2: tokens_reverse.get("EQ2").unwrap().0,
            neq: tokens_reverse.get("NEQ").unwrap().0,
            and: tokens_reverse.get("AND").unwrap().0,
            or: tokens_reverse.get("OR").unwrap().0,
            not: tokens_reverse.get("NOT").unwrap().0,
            uminus: tokens_reverse.get("UMINUS").unwrap().0,
            sharp: tokens_reverse.get("SHARP").unwrap().0,
            lparenfunc: tokens_reverse.get("LPARENFUNC").unwrap().0,
            rparenfunc: tokens_reverse.get("RPARENFUNC").unwrap().0,
            semifield: tokens_reverse.get("SEMIFIELD").unwrap().0,
            questionmark: tokens_reverse.get("QUESTIONMARK").unwrap().0,
            struct_t: tokens_reverse.get("STRUCT").unwrap().0,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum FernLexerState {
    Start,
    InFunctionDef(bool, u32),
    InString,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub enum FernData {
    Number(i64),
    String(String),
    NoData
}

enum InLiteral {
    Name,
    Number,
    None,
}

impl Default for FernData {
    fn default() -> Self {
        NoData
    }
}

pub struct FernLexer {
    pub tokens: Vec<(Token, FernData)>,
    pub data: HashMap<usize, String>,
    pub state: FernLexerState,
    in_literal: InLiteral,
    buf: String,
    grammar: OpGrammar,
    tok: FernTokens,
}

impl LexerInterface<FernLexerState, FernData> for FernLexer {
    fn new(grammar: OpGrammar, start_state: FernLexerState) -> Self {
        FernLexer {
            tokens: Vec::new(),
            state: start_state,
            // This is fine so long as each state transition starts and ends in the same state always.
            in_literal: InLiteral::None,
            buf: String::new(),
            data: HashMap::new(),
            tok: FernTokens::new(&grammar.token_reverse),
            grammar,
        }
    }

    fn consume(&mut self, c: &u8) -> Result<(), LexerError> {
        let c = *c as char;
        loop {
            let mut should_reconsume = false;

            match self.state {
                Start => {
                    match self.in_literal {
                        Name => {
                            match c {
                                'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                                    self.buf.push(c);
                                },
                                _ => {
                                    let token = match self.buf.as_str() {
                                        "and" => self.tok.and,
                                        "struct" => self.tok.struct_t,
                                        "break" => self.tok.break_t,
                                        "else" => self.tok.else_t,
                                        "elseif" => self.tok.elseif,
                                        "false" => self.tok.false_t,
                                        "goto" => self.tok.goto,
                                        "if" => self.tok.if_t,
                                        "nil" => self.tok.nil,
                                        "not" => self.tok.not,
                                        "or" => self.tok.or,
                                        "repeat" => self.tok.repeat,
                                        "until" => self.tok.until,
                                        "in" => self.tok.in_t,
                                        "for" => self.tok.for_t,
                                        "return" => self.tok.return_t,
                                        "then" => self.tok.then,
                                        "true" => self.tok.true_t,
                                        "while" => self.tok.while_t,
                                        "let" => self.tok.let_t,
                                        "fn" => self.tok.fn_t,
                                        "end" => self.tok.end,
                                        _ => self.tok.name,
                                    };
                                    if token == self.tok.name {
                                        self.push(token, FernData::String(self.buf.clone()));
                                    } else {
                                        self.push(token, NoData);
                                    }
                                    if token == self.tok.fn_t {
                                        self.state = InFunctionDef(false, 0);
                                    }
                                    self.in_literal = InLiteral::None;
                                    self.buf.clear();
                                    should_reconsume = true;
                                }
                            }
                        }
                        Number => {
                            match c {
                                '0'..='9' => self.buf.push(c),
                                _ => {
                                    self.push(self.tok.number, FernData::Number(self.buf.parse().unwrap()));
                                    self.buf.clear();
                                    self.in_literal = InLiteral::None;
                                    should_reconsume = true;
                                }
                            }
                        }
                        InLiteral::None => {
                            match c {
                                'a'..='z' | 'A'..='Z' | '_' => {
                                    self.buf.push(c);
                                    self.in_literal = Name;
                                }
                                '0'..='9' => {
                                    self.buf.push(c);
                                    self.in_literal = Number;
                                },
                                '{' => self.push(self.tok.lbrace, NoData),
                                '}' => self.push(self.tok.rbrace, NoData),
                                '[' => self.push(self.tok.lbrack, NoData),
                                ']' => self.push(self.tok.rbrack, NoData),
                                '(' => self.push(self.tok.lparen, NoData),
                                ')' => self.push(self.tok.rparen, NoData),
                                '?' => self.push(self.tok.questionmark, NoData),
                                '.' => self.push(self.tok.dot, NoData),
                                ':' => self.push(self.tok.colon, NoData),
                                ',' => self.push(self.tok.comma, NoData),
                                ';' => self.push(self.tok.semi, NoData),
                                '+' => self.push(self.tok.plus, NoData),
                                '-' => self.push(self.tok.uminus, NoData),
                                '*' => self.push(self.tok.asterisk, NoData),
                                '/' => self.push(self.tok.divide, NoData),
                                '>' => self.push(self.tok.gt, NoData),
                                '<' => self.push(self.tok.lt, NoData),
                                '=' => self.push(self.tok.eq, NoData),
                                '\"' => {
                                    self.state = FernLexerState::InString;
                                }
                                '\n' => {},
                                ' ' | '\t' => {}
                                _ => {
                                    return Err(LexerError::from(format!("Unrecognized char consumed by lexer '{}'", c)));
                                }
                            }
                        }
                    }
                },
                FernLexerState::InString => match c {
                    '\"' => {
                        self.state = Start;
                        self.push(self.tok.string, FernData::String(self.buf.clone()));
                        self.buf.clear();
                    }
                    '\n' => {
                        return Err(LexerError::from("Cannot have newlines in strings".to_string()));
                    }
                    _ => self.buf.push(c),
                },
                InFunctionDef(has_encountered_lparen, mut paren_cnt) => {
                    match self.in_literal {
                        Name => {
                            match c {
                                'a'..='z' | 'A'..='Z' | '_' => {
                                    self.buf.push(c);
                                }
                                _ => {
                                    let token = match self.buf.as_str() {
                                        "and" => self.tok.and,
                                        "struct" => self.tok.struct_t,
                                        "break" => self.tok.break_t,
                                        "else" => self.tok.else_t,
                                        "elseif" => self.tok.elseif,
                                        "false" => self.tok.false_t,
                                        "goto" => self.tok.goto,
                                        "if" => self.tok.if_t,
                                        "nil" => self.tok.nil,
                                        "not" => self.tok.not,
                                        "or" => self.tok.or,
                                        "repeat" => self.tok.repeat,
                                        "until" => self.tok.until,
                                        "in" => self.tok.in_t,
                                        "for" => self.tok.for_t,
                                        "return" => self.tok.return_t,
                                        "then" => self.tok.then,
                                        "true" => self.tok.true_t,
                                        "while" => self.tok.while_t,
                                        "let" => self.tok.let_t,
                                        "fn" => self.tok.fn_t,
                                        "end" => self.tok.end,
                                        _ => self.tok.name,
                                    };
                                    if token == self.tok.name {
                                        self.push(token, FernData::String(self.buf.clone()));
                                    } else {
                                        self.push(token, NoData);
                                    }
                                    if token == self.tok.fn_t {
                                        panic!("function def inside function def?? user error for sure...");
                                    }
                                    self.in_literal = InLiteral::None;
                                    self.buf.clear();
                                    should_reconsume = true;
                                }
                            }
                        }
                        Number => {
                            match c {
                                '0'..='9' => self.buf.push(c),
                                _ => {
                                    self.push(self.tok.number, FernData::Number(self.buf.parse().unwrap()));
                                    self.buf.clear();
                                    self.in_literal = InLiteral::None;
                                    should_reconsume = true;
                                }
                            }
                        }
                        InLiteral::None => {
                            match c {
                                'a'..='z' | 'A'..='Z' | '_' => {
                                    self.buf.push(c);
                                    self.in_literal = Name;
                                }
                                '0'..='9' => {
                                    self.buf.push(c);
                                    self.in_literal = Number;
                                },
                                '{' => self.push(self.tok.lbrace, NoData),
                                '}' => self.push(self.tok.rbrace, NoData),
                                '[' => self.push(self.tok.lbrack, NoData),
                                ']' => self.push(self.tok.rbrack, NoData),
                                '(' => {
                                    if has_encountered_lparen && paren_cnt == 0 {
                                        self.push(self.tok.lparen, NoData);
                                    } else if has_encountered_lparen && paren_cnt != 0{
                                        self.push(self.tok.lparen, NoData);
                                        self.state = InFunctionDef(has_encountered_lparen, paren_cnt + 1);
                                    } else if !has_encountered_lparen {
                                        self.push(self.tok.lparenfunc, NoData);
                                        self.state = InFunctionDef(true, paren_cnt + 1);
                                    }
                                },
                                ')' => {
                                    if has_encountered_lparen && paren_cnt == 1 {
                                        self.push(self.tok.rparenfunc, NoData);
                                        self.state = Start;
                                    } else if has_encountered_lparen && paren_cnt == 0 {
                                        self.push(self.tok.rparen, NoData);
                                    } else if has_encountered_lparen && paren_cnt > 1 {
                                        self.push(self.tok.rparen, NoData);
                                        self.state = InFunctionDef(has_encountered_lparen, paren_cnt - 1);
                                    } else if !has_encountered_lparen {
                                        self.push(self.tok.rparen, NoData);
                                    }
                                },
                                '?' => self.push(self.tok.questionmark, NoData),
                                '.' => self.push(self.tok.dot, NoData),
                                ':' => self.push(self.tok.colon, NoData),
                                ',' => self.push(self.tok.comma, NoData),
                                ';' => self.push(self.tok.semi, NoData),
                                '+' => self.push(self.tok.plus, NoData),
                                '-' => self.push(self.tok.uminus, NoData),
                                '*' => self.push(self.tok.asterisk, NoData),
                                '/' => self.push(self.tok.divide, NoData),
                                '>' => self.push(self.tok.gt, NoData),
                                '<' => self.push(self.tok.lt, NoData),
                                '=' => self.push(self.tok.eq, NoData),
                                '\"' => {
                                    self.state = FernLexerState::InString;
                                }
                                '\n' => {},
                                ' ' | '\t' => {}
                                _ => {
                                    return Err(LexerError::from(format!("Unrecognized char consumed by lexer '{}'", c)));
                                }
                            }
                        }
                    }
                }
            }

            if !should_reconsume {
                break;
            }
        }
        return Ok(());
    }
    fn take(self) -> (FernLexerState, Vec<(Token, FernData)>) {
        (self.state, self.tokens)
    }
}

impl FernLexer {
    fn push(&mut self, t: Token, d: FernData) {
        self.tokens.push((t,d));
        if self.tokens.len() >= 2 {
            if let Some(second_last) = self.tokens.get(self.tokens.len() - 2) {
                if let Some(last) = self.tokens.last() {
                    if self.should_insert_semi(&second_last.0, &last.0) {
                        self.tokens.insert(self.tokens.len() - 1, (self.tok.semi, NoData));
                        trace!("{}", self.grammar.token_raw.get(&self.tok.semi).unwrap());
                    }
                }
            }
        }
        trace!("{}", self.grammar.token_raw.get(&t).unwrap());
    }

    fn should_insert_semi(&self, first: &Token, second: &Token) -> bool {
        let first = *first;
        if first  == self.tok.rbrace ||
            first  == self.tok.rparen ||
            first  == self.tok.rbrack ||
            first  == self.tok.true_t ||
            first  == self.tok.false_t ||
            first  == self.tok.nil ||
            first  == self.tok.number ||
            first  == self.tok.string
            // || first  == self.tok.name
        {
            if *second == self.tok.lparen ||
                *second == self.tok.name ||
                *second == self.tok.break_t ||
                *second == self.tok.if_t ||
                *second == self.tok.while_t ||
                *second == self.tok.let_t ||
                *second == self.tok.return_t ||
                *second == self.tok.for_t ||
                *second == self.tok.fn_t {
                return true;
            }
        }
        return false;
    }
}
