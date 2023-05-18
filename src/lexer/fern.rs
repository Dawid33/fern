use crate::grammar::reader::TokenTypes;
use crate::grammar::{OpGrammar, Token};
use crate::lexer::error::LexerError;
use crate::lexer::LexerInterface;
use log::trace;
use std::collections::HashMap;
use std::fmt::Debug;
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
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum FernLexerState {
    Start,
    InString,
    InName,
    InNumber,
}

pub struct FernLexer {
    pub tokens: Vec<Token>,
    pub data: HashMap<usize, String>,
    pub state: FernLexerState,
    buf: String,
    grammar: OpGrammar,
    tok: FernTokens,
}

impl LexerInterface<FernLexerState> for FernLexer {
    fn new(grammar: OpGrammar, start_state: FernLexerState) -> Self {
        FernLexer {
            tokens: Vec::new(),
            state: start_state,
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
                FernLexerState::Start => match c {
                    'a'..='z' | 'A'..='Z' => {
                        self.state = FernLexerState::InName;
                        self.buf.push(c);
                    }
                    '{' => self.push(self.tok.lbrace),
                    '}' => self.push(self.tok.rbrace),
                    '[' => self.push(self.tok.lbrack),
                    ']' => self.push(self.tok.rbrack),
                    '(' => self.push(self.tok.lparen),
                    ')' => self.push(self.tok.rparen),
                    '.' => self.push(self.tok.dot),
                    ':' => self.push(self.tok.colon),
                    ',' => self.push(self.tok.comma),
                    ';' => self.push(self.tok.semi),
                    '+' => self.push(self.tok.plus),
                    '-' => self.push(self.tok.uminus),
                    '*' => self.push(self.tok.asterisk),
                    '/' => self.push(self.tok.divide),
                    '>' => self.push(self.tok.gt),
                    '<' => self.push(self.tok.lt),
                    '=' => self.push(self.tok.eq),
                    '\"' => {
                        self.state = FernLexerState::InString;
                    }
                    '0'..='9' => {
                        self.state = FernLexerState::InNumber;
                        self.buf.push(c);
                    }
                    '\n' => {},
                    ' ' | '\t' => {}
                    _ => {
                        return Err(LexerError::from(format!("Unrecognized char consumed by lexer '{}'", c)));
                    }
                },
                FernLexerState::InString => match c {
                    '\"' => {
                        self.state = FernLexerState::Start;
                        self.buf.clear();
                        self.push(self.tok.string);
                    }
                    '\n' => {
                        return Err(LexerError::from("Cannot have newlines in strings".to_string()));
                    }
                    _ => self.buf.push(c),
                },
                FernLexerState::InNumber => match c {
                    '0'..='9' => self.buf.push(c),
                    _ => {
                        self.state = FernLexerState::Start;
                        self.push(self.tok.number);
                        self.data.insert(self.tokens.len(), self.buf.clone());
                        self.buf.clear();
                        should_reconsume = true;
                    }
                },
                FernLexerState::InName => match c {
                    'a'..='z' | 'A'..='Z' | '_' => {
                        self.buf.push(c);
                    }
                    _ => {
                        self.state = FernLexerState::Start;
                        let token = match self.buf.as_str() {
                            "and" => self.tok.and,
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
                        self.buf.clear();
                        self.push(token);
                        should_reconsume = true
                    }
                },
            }

            if !should_reconsume {
                break;
            }
        }
        return Ok(());
    }
    fn take(self) -> (FernLexerState, Vec<Token>) {
        (self.state, self.tokens)
    }
}

impl FernLexer {
    fn push(&mut self, t: Token) {
        self.tokens.push(t);
        if self.tokens.len() >= 2 {
            if let Some(second_last) = self.tokens.get(self.tokens.len() - 2) {
                if let Some(last) = self.tokens.last() {
                    if self.should_insert_semi(second_last, last) {
                        self.tokens.insert(self.tokens.len() - 1, self.tok.semi);
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
            first  == self.tok.string ||
            first  == self.tok.name {
            if *second == self.tok.lparen ||
                *second == self.tok.name ||
                *second == self.tok.break_t ||
                *second == self.tok.if_t ||
                *second == self.tok.elseif ||
                *second == self.tok.while_t ||
                *second == self.tok.let_t ||
                *second == self.tok.for_t ||
                *second == self.tok.fn_t {
                return true;
            }
        }
        return false;
    }
}
