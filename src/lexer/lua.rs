use crate::grammar::reader::TokenTypes;
use crate::grammar::Grammar;
use crate::lexer::error::LexerError;
use crate::lexer::LexerInterface;
use log::trace;
use std::collections::HashMap;
use std::fmt::Debug;

pub struct LuaTokens {
    pub endfile: u8,
    pub return_t: u8,
    pub semi: u8,
    pub colon: u8,
    pub colon2: u8,
    pub dot: u8,
    pub dot3: u8,
    pub comma: u8,
    pub lbrack: u8,
    pub rbrack: u8,
    pub lbrace: u8,
    pub rbrace: u8,
    pub lparen: u8,
    pub rparen: u8,
    pub eq: u8,
    pub break_t: u8,
    pub goto: u8,
    pub while_t: u8,
    pub if_t: u8,
    pub then: u8,
    pub elseif: u8,
    pub nil: u8,
    pub else_t: u8,
    pub false_t: u8,
    pub true_t: u8,
    pub number: u8,
    pub string: u8,
    pub name: u8,
    pub plus: u8,
    pub minus: u8,
    pub asterisk: u8,
    pub divide: u8,
    pub caret: u8,
    pub percent: u8,
    pub dot2: u8,
    pub lt: u8,
    pub gt: u8,
    pub lteq: u8,
    pub gteq: u8,
    pub eq2: u8,
    pub neq: u8,
    pub and: u8,
    pub or: u8,
    pub not: u8,
    pub uminus: u8,
    pub sharp: u8,
    pub lparenfunc: u8,
    pub rparenfunc: u8,
    pub semifield: u8,
    pub xeq: u8,
    pub local: u8,
    pub function: u8,
}

impl LuaTokens {
pub fn new(tokens_reverse: &HashMap<String, (u8, TokenTypes)>) -> Self {
        LuaTokens {
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
            while_t: tokens_reverse.get("WHILE").unwrap().0,
            if_t: tokens_reverse.get("IF").unwrap().0,
            then: tokens_reverse.get("THEN").unwrap().0,
            elseif: tokens_reverse.get("ELSEIF").unwrap().0,
            nil: tokens_reverse.get("NIL").unwrap().0,
            else_t: tokens_reverse.get("ELSE").unwrap().0,
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
            xeq: tokens_reverse.get("XEQ").unwrap().0,
            local: tokens_reverse.get("LOCAL").unwrap().0,
            function: tokens_reverse.get("FN").unwrap().0,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum LuaLexerState {
    Start,
    InString,
    InName,
    InNumber,
}

pub struct LuaLexer {
    pub tokens: Vec<u8>,
    pub data: HashMap<usize, String>,
    pub state: LuaLexerState,
    buf: String,
    grammar: Grammar,
    tok: LuaTokens,
}

impl LexerInterface<LuaLexerState> for LuaLexer {
    fn new(grammar: Grammar, start_state: LuaLexerState) -> Self {
        FernLexer {
            tokens: Vec::new(),
            state: start_state,
            buf: String::new(),
            data: HashMap::new(),
            tok: FernTokens::new(&grammar.tokens_reverse),
            grammar,
        }
    }

    fn consume(&mut self, c: &u8) -> Result<(), LexerError> {
        loop {
            let mut should_reconsume = false;

            let c = *c as char;
            let mut push = |t: u8| {
                trace!("{:?}", self.grammar.token_raw.get(&t).unwrap());
                self.tokens.push(t);
            };

            match self.state {
                FernLexerState::Start => match c {
                    'a'..='z' | 'A'..='Z' => {
                        self.state = FernLexerState::InName;
                        self.buf.push(c);
                    }
                    '{' => push(self.tok.lbrace),
                    '}' => push(self.tok.rbrace),
                    '(' => push(self.tok.lparen),
                    ')' => push(self.tok.lparen),
                    ':' => push(self.tok.colon),
                    ',' => push(self.tok.comma),
                    ';' => push(self.tok.semi),
                    '>' => push(self.tok.gt),
                    '<' => push(self.tok.lt),
                    '=' => push(self.tok.xeq),
                    '-' => push(self.tok.minus),
                    '\"' => {
                        self.state = FernLexerState::InString;
                    }
                    '0'..='9' => {
                        self.state = FernLexerState::InNumber;
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
                FernLexerState::InString => match c {
                    '\"' => {
                        self.state = FernLexerState::Start;
                        self.buf.clear();
                        push(self.tok.string);
                    }
                    '\n' => {
                        return Err(LexerError::from(
                            "Cannot have newlines in strings".to_string(),
                        ));
                    }
                    _ => self.buf.push(c),
                },
                FernLexerState::InNumber => match c {
                    '0'..='9' => self.buf.push(c),
                    _ => {
                        self.state = FernLexerState::Start;
                        push(self.tok.number);
                        self.data.insert(self.tokens.len(), self.buf.clone());
                        self.buf.clear();
                        should_reconsume = true;
                    }
                },
                FernLexerState::InName => match c {
                    'a'..='z' | 'A'..='Z' | '_' => {
                        self.buf.push(c);
                    }
                    '\n' | ' ' | '\t' | ';' | ',' | '(' => {
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
                            "return" => self.tok.return_t,
                            "then" => self.tok.then,
                            "true" => self.tok.true_t,
                            "while" => self.tok.while_t,
                            "local" => self.tok.local,
                            "function" => self.tok.function,
                            _ => self.tok.name,
                        };
                        self.buf.clear();
                        push(token);
                        if c == ';' || c == ',' || c == '(' {should_reconsume = true};
                    }
                    _ => {
                        return Err(LexerError::from(
                            "Cannot have newlines in strings".to_string(),
                        ));
                    }
                },
            }

            if !should_reconsume {
                break;
            }
        }
        return Ok(());
    }
    fn take(self) -> (FernLexerState, Vec<u8>) {
        (self.state, self.tokens)
    }
}
