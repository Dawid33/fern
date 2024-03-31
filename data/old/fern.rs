use crate::grammar::reader::TokenTypes;
use crate::grammar::{OpGrammar, Token};
use crate::lexer::error::LexerError;
use crate::lexer::fern::FernData::NoData;
use crate::lexer::fern::FernLexerState::{InFunctionDef, Start};
use crate::lexer::fern::InLiteral::{Name, Number};
use crate::lexer::lua::LuaLexer;
use crate::lexer::LexerInterface;
use log::trace;
use std::collections::HashMap;
use std::fmt::Debug;

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

    pub n_stat_list: Token,
    pub n_stat: Token,
    pub n_else_if_block: Token,
    pub n_expr_then_else_if_b: Token,
    pub n_expr_then: Token,
    pub n_name: Token,
    pub n_ret_stat: Token,
    pub n_label: Token,
    pub n_func_name: Token,
    pub n_name_dot_list: Token,
    pub var_list: Token,
    pub var: Token,
    pub name_list: Token,
    pub expr_list: Token,
    pub expr: Token,
    pub logical_or_exp: Token,
    pub logical_and_exp: Token,
    pub relational_exp: Token,
    pub concat_exp: Token,
    pub additive_exp: Token,
    pub multiplicative_exp: Token,
    pub unary_exp: Token,
    pub caret_exp: Token,
    pub base_exp: Token,
    pub n_prefix_exp: Token,
    pub n_function_call: Token,
    pub n_function_def: Token,
    pub n_par_list: Token,
    pub n_table_constructor: Token,
    pub n_field_list: Token,
    pub n_field_list_body: Token,
    pub n_field: Token,
    pub n_type_expr: Token,
    pub n_ptr_type_start: Token,
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
            eq2: tokens_reverse.get("EQDOUBLE").unwrap().0,
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
            n_stat_list: tokens_reverse.get("statList").unwrap().0,
            n_stat: tokens_reverse.get("stat").unwrap().0,
            n_else_if_block: tokens_reverse.get("elseIfBlock").unwrap().0,
            n_expr_then_else_if_b: tokens_reverse.get("exprThenElseIfB").unwrap().0,
            n_expr_then: tokens_reverse.get("exprThen").unwrap().0,
            n_name: tokens_reverse.get("name").unwrap().0,
            n_ret_stat: tokens_reverse.get("retStat").unwrap().0,
            n_label: tokens_reverse.get("label").unwrap().0,
            n_func_name: tokens_reverse.get("funcName").unwrap().0,
            n_name_dot_list: tokens_reverse.get("nameDotList").unwrap().0,
            var_list: tokens_reverse.get("varList").unwrap().0,
            var: tokens_reverse.get("var").unwrap().0,
            name_list: tokens_reverse.get("nameList").unwrap().0,
            expr_list: tokens_reverse.get("exprList").unwrap().0,
            expr: tokens_reverse.get("expr").unwrap().0,
            logical_or_exp: tokens_reverse.get("logicalOrExp").unwrap().0,
            logical_and_exp: tokens_reverse.get("logicalAndExp").unwrap().0,
            relational_exp: tokens_reverse.get("relationalExp").unwrap().0,
            concat_exp: tokens_reverse.get("concatExp").unwrap().0,
            additive_exp: tokens_reverse.get("additiveExp").unwrap().0,
            multiplicative_exp: tokens_reverse.get("multiplicativeExp").unwrap().0,
            unary_exp: tokens_reverse.get("unaryExp").unwrap().0,
            caret_exp: tokens_reverse.get("caretExp").unwrap().0,
            base_exp: tokens_reverse.get("baseExp").unwrap().0,
            n_prefix_exp: tokens_reverse.get("prefixExp").unwrap().0,
            n_function_call: tokens_reverse.get("functionCall").unwrap().0,
            n_function_def: tokens_reverse.get("functionDef").unwrap().0,
            n_par_list: tokens_reverse.get("parList").unwrap().0,
            n_table_constructor: tokens_reverse.get("tableConstructor").unwrap().0,
            n_field_list: tokens_reverse.get("fieldList").unwrap().0,
            n_field_list_body: tokens_reverse.get("fieldListBody").unwrap().0,
            n_field: tokens_reverse.get("field").unwrap().0,
            n_type_expr: tokens_reverse.get("typeExpr").unwrap().0,
            n_ptr_type_start: tokens_reverse.get("ptrTypeStart").unwrap().0,
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
    NoData,
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
                Start => match self.in_literal {
                    Name => match c {
                        'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
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
                                self.state = InFunctionDef(false, 0);
                            }
                            self.in_literal = InLiteral::None;
                            self.buf.clear();
                            should_reconsume = true;
                        }
                    },
                    Number => match c {
                        '0'..='9' => self.buf.push(c),
                        _ => {
                            self.push(self.tok.number, FernData::Number(self.buf.parse().unwrap()));
                            self.buf.clear();
                            self.in_literal = InLiteral::None;
                            should_reconsume = true;
                        }
                    },
                    InLiteral::None => match c {
                        'a'..='z' | 'A'..='Z' | '_' => {
                            self.buf.push(c);
                            self.in_literal = Name;
                        }
                        '0'..='9' => {
                            self.buf.push(c);
                            self.in_literal = Number;
                        }
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
                        '-' => self.push(self.tok.minus, NoData),
                        '*' => self.push(self.tok.asterisk, NoData),
                        '/' => self.push(self.tok.divide, NoData),
                        '%' => self.push(self.tok.percent, NoData),
                        '>' => self.push(self.tok.gt, NoData),
                        '<' => self.push(self.tok.lt, NoData),
                        '=' => self.push(self.tok.eq, NoData),
                        '\"' => {
                            self.state = FernLexerState::InString;
                        }
                        '\n' => {}
                        ' ' | '\t' => {}
                        _ => {
                            return Err(LexerError::from(format!("Unrecognized char consumed by lexer '{}'", c)));
                        }
                    },
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
                InFunctionDef(has_encountered_lparen, paren_cnt) => match self.in_literal {
                    Name => match c {
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
                    },
                    Number => match c {
                        '0'..='9' => self.buf.push(c),
                        _ => {
                            self.push(self.tok.number, FernData::Number(self.buf.parse().unwrap()));
                            self.buf.clear();
                            self.in_literal = InLiteral::None;
                            should_reconsume = true;
                        }
                    },
                    InLiteral::None => match c {
                        'a'..='z' | 'A'..='Z' | '_' => {
                            self.buf.push(c);
                            self.in_literal = Name;
                        }
                        '0'..='9' => {
                            self.buf.push(c);
                            self.in_literal = Number;
                        }
                        '{' => self.push(self.tok.lbrace, NoData),
                        '}' => self.push(self.tok.rbrace, NoData),
                        '[' => self.push(self.tok.lbrack, NoData),
                        ']' => self.push(self.tok.rbrack, NoData),
                        '(' => {
                            if has_encountered_lparen && paren_cnt == 0 {
                                self.push(self.tok.lparen, NoData);
                            } else if has_encountered_lparen && paren_cnt != 0 {
                                self.push(self.tok.lparen, NoData);
                                self.state = InFunctionDef(has_encountered_lparen, paren_cnt + 1);
                            } else if !has_encountered_lparen {
                                self.push(self.tok.lparenfunc, NoData);
                                self.state = InFunctionDef(true, paren_cnt + 1);
                            }
                        }
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
                        }
                        '?' => self.push(self.tok.questionmark, NoData),
                        '.' => self.push(self.tok.dot, NoData),
                        ':' => self.push(self.tok.colon, NoData),
                        ',' => self.push(self.tok.comma, NoData),
                        ';' => self.push(self.tok.semi, NoData),
                        '+' => self.push(self.tok.plus, NoData),
                        '-' => self.push(self.tok.minus, NoData),
                        '*' => self.push(self.tok.asterisk, NoData),
                        '/' => self.push(self.tok.divide, NoData),
                        '%' => self.push(self.tok.percent, NoData),
                        '>' => self.push(self.tok.gt, NoData),
                        '<' => self.push(self.tok.lt, NoData),
                        '=' => self.push(self.tok.eq, NoData),
                        '\"' => {
                            self.state = FernLexerState::InString;
                        }
                        '\n' => {}
                        ' ' | '\t' => {}
                        _ => {
                            return Err(LexerError::from(format!("Unrecognized char consumed by lexer '{}'", c)));
                        }
                    },
                },
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
        self.tokens.push((t, d));
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
        if first == self.tok.rbrace
            || first == self.tok.rparen
            || first == self.tok.rbrack
            || first == self.tok.true_t
            || first == self.tok.false_t
            || first == self.tok.nil
            || first == self.tok.number
            || first == self.tok.string
        // || first  == self.tok.name
        {
            if *second == self.tok.lparen
                || *second == self.tok.name
                || *second == self.tok.break_t
                || *second == self.tok.if_t
                || *second == self.tok.while_t
                || *second == self.tok.let_t
                || *second == self.tok.return_t
                || *second == self.tok.for_t
                || *second == self.tok.fn_t
            {
                return true;
            }
        }
        return false;
    }
} // pub fn render<W: Write>(ast: Box<AstNode>, output: &mut W) {
  //     let mut nodes: Vec<String> = Vec::new();
  //     let mut edges = VecDeque::new();

//     nodes.push("Module".to_string());
//     let mut stack: Vec<(Box<AstNode>, usize)> = vec![(ast, 0)];

//     while let Some((current, id)) = stack.pop() {
//         let mut push_node = |id, str: String, node: Box<AstNode>| {
//             nodes.push(str);
//             let child = nodes.len() - 1;
//             edges.push_front((id, child));
//             stack.push((node, child));
//         };

//         match *current {
//             AstNode::Binary(left, _, right) => {
//                 push_node(id, format!("{:?}", &left), left);
//                 push_node(id, format!("{:?}", &right), right);
//             }
//             AstNode::Unary(_, expr) => {
//                 push_node(id, format!("{:?}", &expr), expr);
//             }
//             AstNode::Number(_) => {}
//             AstNode::String(_) => {}
//             AstNode::Name(_) => {}
//             AstNode::ExprList(expr_list) => {
//                 for x in expr_list {
//                     push_node(id, format!("{:?}", x), Box::from(x));
//                 }
//             }
//             AstNode::Assign(name, expr) => {
//                 push_node(id, format!("{:?}", name), name);
//                 push_node(id, format!("{:?}", expr), expr);
//             }
//             AstNode::Let(name, _, expr) => {
//                 push_node(id, format!("{:?}", name), name);
//                 if let Some(expr) = expr {
//                     push_node(id, format!("{:?}", expr), expr);
//                 }
//             }
//             AstNode::Module(stmts) => {
//                 push_node(id, format!("{:?}", stmts), stmts);
//             }
//             AstNode::Function(_, param, stmts) => {
//                 if let Some(p) = param {
//                     push_node(id, format!("Parameters: {:?}", p), p);
//                 }
//                 if let Some(stmts) = stmts {
//                     push_node(id, format!("{:?}", stmts), stmts);
//                 }
//             }
//             AstNode::FunctionCall(expr, args) => {
//                 push_node(id, format!("{:?}", expr), expr);
//                 if let Some(args) = args {
//                     push_node(id, format!("{:?}", args), args);
//                 }
//             }
//             AstNode::If(expr, stmts, else_or_elseif) => {
//                 push_node(id, format!("<B>Condition</B><BR/>{:?}", expr), expr);
//                 if let Some(stmts) = stmts {
//                     push_node(id, format!("<B>If Body</B><BR/>{:?}", stmts), stmts);
//                 }
//                 if let Some(e) = else_or_elseif {
//                     push_node(id, format!("{:?}", e), e);
//                 }
//             }
//             AstNode::For(var, expr, stmts) => {
//                 push_node(id, format!("Variable\n{:?}", var), var);
//                 push_node(id, format!("List\n{:?}", expr), expr);
//                 push_node(id, format!("{:?}", stmts), stmts);
//             }
//             AstNode::While(expr, stmts) => {
//                 push_node(id, format!("Condition\n{:?}", expr), expr);
//                 push_node(id, format!("{:?}", stmts), stmts);
//             }
//             AstNode::Return(expr) => {
//                 if let Some(expr) = expr {
//                     push_node(id, format!("{:?}", expr), expr);
//                 }
//             }
//             AstNode::ElseIf(expr, stmts, else_or_elseif) => {
//                 push_node(id, format!("<B>Condition</B><BR/>{:?}", expr), expr);
//                 if let Some(stmts) = stmts {
//                     push_node(id, format!("<B>Else If Body</B><BR/>{:?}", stmts), stmts);
//                 }
//                 if let Some(e) = else_or_elseif {
//                     push_node(id, format!("{:?}", e), e);
//                 }
//             }
//             AstNode::Else(stmts) => {
//                 if let Some(stmts) = stmts {
//                     push_node(id, format!("{:?}", stmts), stmts);
//                 }
//             }
//             AstNode::StatList(stmts) => {
//                 for x in stmts {
//                     push_node(id, format!("{:?}", x), Box::from(x));
//                 }
//             }
//             AstNode::ExprThen(expr, stmt) => {
//                 push_node(id, format!("Condition\n{:?}", expr), expr);
//                 if let Some(e) = stmt {
//                     push_node(id, format!("{:?}", e), e);
//                 }
//             }
//         }
//     }

//     let graph = Graph { nodes, edges };
//     dot::render(&graph, output).unwrap()
// }

// pub fn render_block<W: Write>(ir: Block, w: &mut W) {
//     w.write(
//         r#"
// digraph g {
//     fontname="Helvetica,Arial,sans-serif"
//     node [fontname="Helvetica,Arial,sans-serif"]
//     edge [fontname="Helvetica,Arial,sans-serif"]
//     graph [
//         rankdir = "LR"
//     ];
//     node [
//         fontsize = "16"
//         shape = "ellipse"
//         rankjustify=min
//     ];
//     edge [
//     ];
//     "root" [
//     label = "root"
//     shape = "record"
//     ];
// "#
//         .as_bytes(),
//     )
//     .unwrap();
//     let mut stack = VecDeque::new();
//     stack.push_front(&ir);
//     let print_b = |b: &Block, w: &mut W| {
//         let mut builder = String::new();
//         let label = match &b.block_type {
//             BlockType::Code(ref stmts) => {
//                 let mut stmt_to_string = |x: &Statement, first: bool| {
//                     let mut prefix = "| ";
//                     if first {
//                         prefix = "";
//                     }
//                     match x {
//                         crate::ir::Statement::Return(val) => {
//                             if let Some(ref val) = val {
//                                 builder.push_str(format!("{}return {}\\l", prefix, val).as_str())
//                             } else {
//                                 builder.push_str(format!("{}return\\l", prefix).as_str())
//                             }
//                         }
//                         crate::ir::Statement::Let(l) => {
//                             if let Some(ref val) = l.val {
//                                 builder.push_str(format!("{}{} = {} \\l", prefix, l.ident.name, val).as_str())
//                             } else {
//                                 builder.push_str(format!("{}{}; \\l", prefix, l.ident.name).as_str())
//                             }
//                         }
//                         _ => builder.push_str("?"),
//                     }
//                 };
//                 let mut stmts = stmts.iter();
//                 if let Some(x) = stmts.next() {
//                     stmt_to_string(&x, true);
//                 };
//                 while let Some(x) = stmts.next() {
//                     stmt_to_string(&x, false);
//                 }
//                 &builder
//             }
//             BlockType::If(cond) => {
//                 builder = format!("{} ({})", &b.prefix, cond);
//                 &builder
//             }
//             _ => {
//                 println!("{}", &b.prefix);
//                 &b.prefix
//             }
//         };

//         w.write(format!("\"{}\" [label = \"{}\"\n shape = \"record\"];\n", b.prefix, label).as_bytes())
//             .unwrap();
//     };

//     while !stack.is_empty() {
//         let current = stack.pop_front().unwrap();
//         print_b(current, w);

//         for b in &current.children {
//             stack.push_front(b);
//             w.write(format!("\"{}\" -> \"{}\" [];\n", current.prefix, b.prefix).as_bytes()).unwrap();
//         }
//     }

//     w.write("\n}\n".as_bytes()).unwrap();
// }

// use crate::grammar::{OpGrammar, Token};
// use crate::lexer::fern::{FernData, FernTokens};
// use crate::parser::fern_ast::Operator::{Add, Divide, Equal, GreaterThan, GreaterThanOrEqual, LessThan, LessThanOrEqual, Modulo, Multiply, NotEqual, Subtract};
// use crate::parser::{Node, ParseTree};
// use log::info;
// use simple_error::SimpleError;
// use std::borrow::Cow;
// use std::cmp::max;
// use std::collections::{HashMap, VecDeque};
// use std::error::Error;
// use std::fmt::{Debug, Formatter};
// use std::io::Write;
// use std::os::unix::fs::symlink;
// use std::sync;

// struct Module {}

// impl Module {
//     pub fn from(_: Box<AstNode>) -> Self {
//         println!("Hello, World");
//         return Self {};
//     }
// }
