use crate::grammar::lg::{self, LexingTable, LookupResult, State};
use crate::grammar::opg::{OpGrammar, RawGrammar, Token};
use crate::lexer::{Data, LexerError, LexerInterface, ParallelLexer};
use crate::parser::{Node, ParallelParser, ParseTree};
use log::{info, warn};
use memmap::MmapOptions;
use simple_error::SimpleError;
use std::borrow::Cow;
use std::cmp::max;
use std::collections::{HashMap, LinkedList, VecDeque};
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::{Read, Write};
use std::time::{Duration, Instant};
use std::{sync, thread};

pub struct FernParseTree {
    pub g: OpGrammar,
    pub root: Node,
}

pub fn compile() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    let first_lg = Instant::now();
    let mut file = File::open("data/grammar/fern.lg").unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    let g = lg::LexicalGrammar::from(buf.clone());
    let nfa = lg::StateGraph::from(g.clone());
    let mut f = File::create("nfa.dot").unwrap();
    lg::render(&nfa, &mut f);
    let dfa = nfa.convert_to_dfa();
    let mut f = File::create("dfa.dot").unwrap();
    lg::render(&dfa, &mut f);
    let mut table = dfa.build_table();
    table.terminal_map.push("UMINUS".to_string());
    table.terminal_map.push("LPARENFUNC".to_string());
    table.terminal_map.push("RPARENFUNC".to_string());
    let first_lg = first_lg.elapsed();
    buf.clear();

    let second_lg = Instant::now();
    let mut file = File::open("data/grammar/keywords.lg").unwrap();
    file.read_to_string(&mut buf).unwrap();
    let g = lg::LexicalGrammar::from(buf.clone());
    let nfa = lg::StateGraph::from(g.clone());
    let mut f = File::create("nfa.dot").unwrap();
    lg::render(&nfa, &mut f);
    let dfa = nfa.convert_to_dfa();
    let mut f = File::create("dfa.dot").unwrap();
    lg::render(&dfa, &mut f);
    let keywords = dfa.build_table();
    let second_lg = second_lg.elapsed();

    let name_token = table.terminal_map.iter().position(|x| x == "NAME").unwrap();
    warn!("{}", name_token);
    table.add_table(name_token, keywords);

    let lex_time = Instant::now();
    let tokens: LinkedList<(Vec<Token>, Vec<Data>)> = {
        let file = File::open("data/test.fern")?;
        let mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };
        thread::scope(|s| {
            let mut lexer: ParallelLexer<FernLexer> = ParallelLexer::new(table.clone(), s, 1);
            let batch = lexer.new_batch();
            lexer.add_to_batch(&batch, &mmap[..], 0);
            let tokens = lexer.collect_batch(batch);
            lexer.kill();
            tokens
        })
    };
    let lex_time = lex_time.elapsed();

    info!("{:?}", table.terminal_map);
    info!("{:?}", &tokens);
    for (l, _) in &tokens {
        for t in l {
            info!("{}", table.terminal_map[*t]);
        }
    }

    let grammar_time = Instant::now();
    let mut raw = RawGrammar::from("data/grammar/fern.g", table.terminal_map).unwrap();
    raw.delete_repeated_rhs()?;
    let grammar = OpGrammar::new(raw)?;
    let grammar_time = grammar_time.elapsed();
    grammar.to_file("data/grammar/fern-fnf.g");

    let parse_time = Instant::now();
    let (tree, time): (ParseTree, Duration) = {
        let mut parser = ParallelParser::new(grammar.clone(), 1);
        parser.parse(tokens);
        parser.parse(LinkedList::from([(vec![grammar.delim], Vec::new())]));
        let time = parser.time_spent_rule_searching.clone();
        (parser.collect_parse_tree().unwrap(), time)
    };
    let parse_time = parse_time.elapsed();

    tree.print();
    info!("Time to build first lexical grammar: {:?}", first_lg);
    info!("Time to build second lexical grammar: {:?}", second_lg);
    info!("Time to lex: {:?}", lex_time);
    info!("Time to build parsing grammar: {:?}", grammar_time);
    info!("Time to parse: {:?}", parse_time);
    info!("└─Time spent rule-searching: {:?}", time);
    info!("Total run time : {:?}", start.elapsed());

    // let ast: Box<AstNode> = Box::from(tree.build_ast().unwrap());
    // info!("Total Time to transform ParseTree -> AST: {:?}", now.elapsed());
    // let mut f = File::create("ast.dot").unwrap();
    // render(ast.clone(), &mut f);

    // now = Instant::now();
    // analysis::check_used_before_declared(ast);
    // info!("Total Time to Analyse AST : {:?}", now.elapsed());

    Ok(())
}

pub struct FernLexer {
    pub table: LexingTable,
    pub start_state: State,
    pub state: State,
    pub buf: String,
    pub tokens: Vec<Token>,
    pub data: Vec<Data>,
    pub whitespace_token: Token,
    had_lparenfunc: i32,
    had_whitespace: bool,
    minus: Token,
    unary_minus: Token,
    name_token: Token,
    lparen: Token,
    rparen: Token,
    lparenfunc: Token,
    rparenfunc: Token,
}

impl LexerInterface for FernLexer {
    fn new(table: LexingTable, start_state: usize) -> Self {
        let name_token = table.terminal_map.iter().position(|x| x == "NAME").unwrap();
        let lparen = table.terminal_map.iter().position(|x| x == "LPAREN").unwrap();
        let rparen = table.terminal_map.iter().position(|x| x == "RPAREN").unwrap();
        let lparenfunc = table.terminal_map.iter().position(|x| x == "LPARENFUNC").unwrap();
        let rparenfunc = table.terminal_map.iter().position(|x| x == "RPARENFUNC").unwrap();
        let whitespace_token = table.terminal_map.iter().position(|x| x == "WHITESPACE").unwrap();
        let minus = table.terminal_map.iter().position(|x| x == "MINUS").unwrap();
        let unary_minus = table.terminal_map.iter().position(|x| x == "UMINUS").unwrap();
        Self {
            table,
            whitespace_token,
            unary_minus,
            minus,
            had_whitespace: false,
            had_lparenfunc: -1,
            name_token,
            lparenfunc,
            rparenfunc,
            lparen,
            rparen,
            tokens: Vec::new(),
            start_state,
            buf: String::new(),
            state: start_state,
            data: Vec::new(),
        }
    }
    fn consume(&mut self, input: u8) -> Result<(), LexerError> {
        let mut reconsume = true;
        while reconsume {
            reconsume = false;
            let result = self.table.get(input, self.state);
            match result {
                LookupResult::Terminal(mut t) => {
                    if let Some((table, offset)) = self.table.sub_tables.get(&t) {
                        let mut state = 0;
                        let buf = format!("{}", self.buf);
                        for c in buf.chars() {
                            match table.get(c as u8, state) {
                                LookupResult::Terminal(token) => {
                                    t = token + offset;
                                    break;
                                }
                                LookupResult::State(s) => {
                                    state = s;
                                }
                                LookupResult::Err => break,
                            }
                        }
                        if let Some(token) = table.try_get_terminal(state) {
                            t = token + offset;
                        }
                    }

                    if t == self.lparen && self.had_lparenfunc >= 0 {
                        self.had_lparenfunc += 1;
                    }

                    if t == self.rparen && self.had_lparenfunc >= 0 {
                        if self.had_lparenfunc == 0 {
                            t = self.rparenfunc;
                        }
                        self.had_lparenfunc -= 1;
                    }

                    info!("c, t: {}, {}", input as char, self.table.terminal_map[t]);
                    if t != self.whitespace_token {
                        let t2 = if self.had_whitespace { &self.whitespace_token } else { &t };
                        if let Some((t1, t2)) = self.look_ahead(*t2) {
                            *self.tokens.last_mut().unwrap() = t1;
                            t = t2;
                        }
                        self.tokens.push(t);
                        self.data.push(Data {
                            token_index: self.tokens.len() - 1,
                            raw: self.buf.clone(),
                        });
                        self.had_whitespace = false;
                    } else {
                        self.had_whitespace = true;
                    }
                    self.buf.clear();
                    self.state = 0;
                    reconsume = true;
                }
                LookupResult::State(s) => {
                    self.buf.push(input as char);
                    self.state = s;
                }
                LookupResult::Err => {
                    warn!("Lexing Error when transitioning state. state : {}", self.state);
                }
            }
        }
        return Ok(());
    }
    fn take(self) -> (State, Vec<Token>, Vec<Data>) {
        (self.state, self.tokens, self.data)
    }
}

impl FernLexer {
    fn look_ahead(&mut self, t2: Token) -> Option<(Token, Token)> {
        if let Some(t1) = self.tokens.last() {
            warn!("look_ahead {}, {}", self.table.terminal_map[*t1], self.table.terminal_map[t2]);
            if *t1 == self.minus && (t2 == self.name_token || t2 == self.lparen) {
                return Some((self.unary_minus, t2));
            }
            if *t1 == self.name_token && t2 == self.lparen {
                self.had_lparenfunc = 0;
                return Some((self.name_token, self.lparenfunc));
            }
        }
        None
    }
}

// #[derive(Clone)]
// pub enum Operator {
//     Add,
//     Multiply,
//     Divide,
//     Modulo,
//     Subtract,
//     Equal,
//     NotEqual,
//     GreaterThan,
//     GreaterThanOrEqual,
//     LessThan,
//     LessThanOrEqual,
// }

// #[derive(Debug, Clone)]
// pub enum TypeExpr {}

// #[derive(Clone)]
// pub enum AstNode {
//     Binary(Box<AstNode>, Operator, Box<AstNode>),
//     Unary(Operator, Box<AstNode>),
//     Number(i64),
//     String(String),
//     Name(String),
//     ExprList(VecDeque<AstNode>),
//     Assign(Box<AstNode>, Box<AstNode>),
//     Let(Box<AstNode>, Option<TypeExpr>, Option<Box<AstNode>>),
//     Return(Option<Box<AstNode>>),
//     Module(Box<AstNode>),
//     StatList(VecDeque<AstNode>),
//     FunctionCall(Box<AstNode>, Option<Box<AstNode>>),
//     Function(Box<AstNode>, Option<Box<AstNode>>, Option<Box<AstNode>>),
//     If(Box<AstNode>, Option<Box<AstNode>>, Option<Box<AstNode>>),
//     ExprThen(Box<AstNode>, Option<Box<AstNode>>),
//     ElseIf(Box<AstNode>, Option<Box<AstNode>>, Option<Box<AstNode>>),
//     Else(Option<Box<AstNode>>),
//     For(Box<AstNode>, Box<AstNode>, Box<AstNode>),
//     While(Box<AstNode>, Box<AstNode>),
// }

// /// Reduce a node of the parse tree into an ast node.
// fn new_reduce<T: Debug>(node: Node<T>, stack: &mut Vec<VecDeque<AstNode>>, tok: &FernTokens, g: &OpGrammar) -> Option<AstNode> {
//     let mut last = if let Some(last) = stack.pop() {
//         last
//     } else {
//         panic!("Cannot reduce an empty stack. Probably finished traversing parse tree too early.");
//     };

//     let reduced: Result<AstNode, SimpleError>;
//     if node.symbol == tok.base_exp {
//         reduced = Ok(last.pop_front().unwrap());
//     } else if node.symbol == tok.n_name {
//         reduced = Ok(last.pop_front().unwrap());
//     } else if node.symbol == tok.additive_exp {
//         reduced = reduce_additive_exp(node, last, tok);
//     } else if node.symbol == tok.multiplicative_exp {
//         reduced = reduce_multiplicative_exp(node, last, tok);
//     } else if node.symbol == tok.relational_exp {
//         reduced = reduce_relational_exp(node, last, tok);
//     } else if node.symbol == tok.n_stat {
//         reduced = reduce_stat(node, last, tok);
//     } else if node.symbol == tok.n_else_if_block {
//         reduced = reduce_else_if(node, last, tok);
//     } else if node.symbol == tok.n_function_call {
//         let expr = last.pop_back().unwrap();
//         let body = last.pop_back();
//         let result = if let Some(b) = body {
//             Ok(AstNode::FunctionCall(Box::from(expr), Some(Box::from(b))))
//         } else {
//             Ok(AstNode::FunctionCall(Box::from(expr), None))
//         };
//         reduced = result
//     } else if node.symbol == tok.n_expr_then {
//         let expr = last.pop_back().unwrap();
//         let body = last.pop_back();
//         let result = if let Some(b) = body {
//             Ok(AstNode::ExprThen(Box::from(expr), Some(Box::from(b))))
//         } else {
//             Ok(AstNode::ExprThen(Box::from(expr), None))
//         };
//         reduced = result
//     } else if node.symbol == tok.n_stat_list {
//         let mut list = VecDeque::new();
//         for x in last {
//             if let AstNode::StatList(child_list) = x {
//                 for x in child_list.into_iter().rev() {
//                     list.push_front(x);
//                 }
//             } else {
//                 list.push_front(x);
//             }
//         }
//         reduced = Ok(AstNode::StatList(list))
//     } else if node.symbol == tok.expr_list {
//         let mut list = VecDeque::new();
//         for x in last {
//             if let AstNode::StatList(child_list) = x {
//                 for x in child_list.into_iter().rev() {
//                     list.push_front(x);
//                 }
//             } else {
//                 list.push_front(x);
//             }
//         }
//         reduced = Ok(AstNode::ExprList(list))
//     } else if node.symbol == tok.n_ret_stat {
//         let exp = last.pop_front();
//         reduced = if let Some(exp) = exp {
//             Ok(AstNode::Return(Some(Box::from(exp))))
//         } else {
//             Ok(AstNode::Return(None))
//         };
//     } else {
//         panic!(
//             "Parse tree node not recognized = {:?}. Probably changed grammar and didn't update ast transform you bad boy.",
//             g.token_raw.get(&node.symbol).unwrap()
//         );
//     }

//     if let Some(parent) = stack.last_mut() {
//         if let Ok(reduced) = reduced {
//             parent.push_back(reduced);
//         }
//     } else if let Ok(reduced) = reduced {
//         // return Some(AstNode::Module(VecDeque::from([reduced])));
//         if let AstNode::StatList(_) = reduced {
//             return Some(AstNode::Module(Box::from(reduced)));
//         } else {
//             return Some(AstNode::Module(Box::from(AstNode::StatList(VecDeque::from_iter([reduced].into_iter())))));
//         }
//     } else {
//         panic!("Cannot reduce, fix buggo.")
//     }
//     None
// }

// fn reduce_additive_exp<T: Debug>(node: Node<T>, mut last: VecDeque<AstNode>, tok: &FernTokens) -> Result<AstNode, SimpleError> {
//     let right = last.pop_front().unwrap();
//     let left = last.pop_front().unwrap();
//     let result = if let Some(op) = node.children.get(0) {
//         let result = if op.symbol == tok.plus {
//             Ok(AstNode::Binary(Box::from(left), Add, Box::from(right)))
//         } else if op.symbol == tok.minus {
//             Ok(AstNode::Binary(Box::from(left), Subtract, Box::from(right)))
//         } else {
//             Err(SimpleError::new("Badly formed additive node in parse tree."))
//         };
//         result
//     } else {
//         Err(SimpleError::new("Badly formed additive node in parse tree."))
//     };
//     result
// }

// fn reduce_multiplicative_exp<T: Debug>(node: Node<T>, mut last: VecDeque<AstNode>, tok: &FernTokens) -> Result<AstNode, SimpleError> {
//     let right = last.pop_front().unwrap();
//     let left = last.pop_front().unwrap();
//     let result = if let Some(op) = node.children.get(0) {
//         let result = if op.symbol == tok.asterisk {
//             Ok(AstNode::Binary(Box::from(left), Multiply, Box::from(right)))
//         } else if op.symbol == tok.divide {
//             Ok(AstNode::Binary(Box::from(left), Divide, Box::from(right)))
//         } else if op.symbol == tok.percent {
//             Ok(AstNode::Binary(Box::from(left), Modulo, Box::from(right)))
//         } else {
//             Err(SimpleError::new("Badly formed multiplicative node in parse tree."))
//         };
//         result
//     } else {
//         Err(SimpleError::new("Badly formed multiplicative node in parse tree."))
//     };
//     result
// }

// fn reduce_relational_exp<T: Debug>(node: Node<T>, mut last: VecDeque<AstNode>, tok: &FernTokens) -> Result<AstNode, SimpleError> {
//     let right = last.pop_front().unwrap();
//     let left = last.pop_front().unwrap();
//     let result = if let Some(op) = node.children.get(0) {
//         let result = if op.symbol == tok.lt {
//             Ok(AstNode::Binary(Box::from(left), LessThan, Box::from(right)))
//         } else if op.symbol == tok.gt {
//             Ok(AstNode::Binary(Box::from(left), GreaterThan, Box::from(right)))
//         } else if op.symbol == tok.lteq {
//             Ok(AstNode::Binary(Box::from(left), LessThanOrEqual, Box::from(right)))
//         } else if op.symbol == tok.gteq {
//             Ok(AstNode::Binary(Box::from(left), GreaterThanOrEqual, Box::from(right)))
//         } else if op.symbol == tok.neq {
//             Ok(AstNode::Binary(Box::from(left), NotEqual, Box::from(right)))
//         } else if op.symbol == tok.eq2 {
//             Ok(AstNode::Binary(Box::from(left), Equal, Box::from(right)))
//         } else {
//             Err(SimpleError::new("Badly formed multiplicative node in parse tree."))
//         };
//         result
//     } else {
//         Err(SimpleError::new("Badly formed multiplicative node in parse tree."))
//     };
//     result
// }

// fn reduce_stat<T: Debug>(node: Node<T>, mut last: VecDeque<AstNode>, tok: &FernTokens) -> Result<AstNode, SimpleError> {
//     let result = if let Some(first) = node.children.first() {
//         let result = if first.symbol == tok.let_t {
//             let exp = last.pop_front().unwrap();
//             let name = last.pop_front();
//             let result = if let Some(name) = name {
//                 Ok(AstNode::Let(Box::from(name), None, Some(Box::from(exp))))
//             } else {
//                 Ok(AstNode::Let(Box::from(exp), None, None))
//             };
//             result
//         } else if first.symbol == tok.if_t {
//             reduce_if(last)
//         } else if first.symbol == tok.fn_t {
//             function(node, last, tok)
//         } else if let Some(first) = last.pop_back() {
//             let result = match first {
//                 AstNode::Name(_) => {
//                     let expr = last.pop_front().unwrap();
//                     Ok(AstNode::Assign(Box::from(first), Box::from(expr)))
//                 }
//                 AstNode::Return(_) => Ok(first),
//                 _ => Err(SimpleError::new("Unkown statement in statement list.")),
//             };
//             result
//         } else {
//             panic!("Either a missing statement parse in ast gen or a bug. Actually its a bug either way.");
//         };
//         result
//     } else {
//         panic!("Either a missing statement parse in ast gen or a bug. Actually its a bug either way.");
//     };

//     result
// }

// fn function<T: Debug>(_: Node<T>, mut last: VecDeque<AstNode>, _: &FernTokens) -> Result<AstNode, SimpleError> {
//     let name = Box::from(last.pop_back().unwrap());
//     let result = if let Some(first) = last.pop_back() {
//         match first {
//             AstNode::ExprList(_) | AstNode::Name(_) => {
//                 let result = if let Some(second) = last.pop_back() {
//                     match second {
//                         AstNode::If(_, _, _) | AstNode::Let(_, _, _) | AstNode::StatList(_) => {
//                             Ok(AstNode::Function(name, Some(Box::from(first)), Some(Box::from(second))))
//                         }
//                         _ => Err(SimpleError::new("Badly formed function definition.")),
//                     }
//                 } else {
//                     Ok(AstNode::Function(name, Some(Box::from(first)), None))
//                 };
//                 result
//             }
//             AstNode::If(_, _, _) | AstNode::Let(_, _, _) | AstNode::StatList(_) => Ok(AstNode::Function(name, None, Some(Box::from(first)))),
//             _ => Err(SimpleError::new("Badly formed function definition.")),
//         }
//     } else {
//         Ok(AstNode::Function(name, None, None))
//     };
//     result
// }

// fn reduce_if(mut last: VecDeque<AstNode>) -> Result<AstNode, SimpleError> {
//     let expr_then = last.pop_back().unwrap();
//     let result = if let AstNode::ExprThen(expr, body) = expr_then {
//         let result = if let Some(else_if_block) = last.pop_front() {
//             Ok(AstNode::If(expr, body, Some(Box::from(else_if_block))))
//         } else {
//             Ok(AstNode::If(expr, body, None))
//         };
//         result
//     } else {
//         Err(SimpleError::new("Badly formed if statement."))
//     };
//     result
// }

// fn reduce_else_if<T: Debug>(node: Node<T>, mut last: VecDeque<AstNode>, tok: &FernTokens) -> Result<AstNode, SimpleError> {
//     let result = if let Some(first) = node.children.first() {
//         let result = if first.symbol == tok.else_t {
//             let result = if let Some(else_block) = last.pop_front() {
//                 Ok(AstNode::Else(Some(Box::from(else_block))))
//             } else {
//                 Ok(AstNode::Else(None))
//             };
//             result
//         } else if first.symbol == tok.elseif {
//             let expr = Box::from(last.pop_back().unwrap());
//             let result = if let Some(first) = last.pop_back() {
//                 match first {
//                     AstNode::StatList(_) => {
//                         let result = if let Some(second) = last.pop_back() {
//                             match second {
//                                 AstNode::ElseIf(_, _, _) | AstNode::Else(_) => Ok(AstNode::ElseIf(expr, Some(Box::from(first)), Some(Box::from(second)))),
//                                 _ => {
//                                     panic!("Badly formed else if / else statement.");
//                                 }
//                             }
//                         } else {
//                             Ok(AstNode::ElseIf(expr, Some(Box::from(first)), None))
//                         };
//                         result
//                     }
//                     AstNode::ElseIf(_, _, _) => Ok(AstNode::ElseIf(expr, None, Some(Box::from(first)))),
//                     _ => {
//                         panic!("Badly formed else if / else statement.");
//                     }
//                 }
//             } else {
//                 Ok(AstNode::ElseIf(expr, None, None))
//             };
//             result
//         } else {
//             panic!("Badly formed else if / else statement.");
//         };
//         result
//     } else {
//         panic!("Badly formed else if statement.");
//     };
//     result
// }

// impl FernParseTree {
//     pub fn build_ast(self) -> Result<AstNode, SimpleError> {
//         let tok = FernTokens::new(&self.g.token_reverse);

//         let mut stack: Vec<VecDeque<AstNode>> = Vec::new();
//         let mut b = String::new();
//         b.push_str(format!("{}", self.g.token_raw.get(&self.root.symbol).unwrap()).as_str());
//         info!("{}", b);
//         b.clear();

//         let mut child_count_stack: Vec<(i32, i32)> = vec![((self.root.children.len() - 1) as i32, 0)];
//         let mut node_stack: Vec<Node<FernData>> = vec![self.root];

//         while !node_stack.is_empty() {
//             let mut current = node_stack.pop().unwrap();
//             let (mut current_child, min_child) = child_count_stack.pop().unwrap();

//             if current.children.len() > 0 && current_child >= min_child {
//                 while current.children.len() > 0 && current_child >= min_child {
//                     for _i in 0..child_count_stack.len() {
//                         b.push_str("  ");
//                     }
//                     b.push_str(
//                         format!(
//                             "{}",
//                             self.g.token_raw.get(&current.children.get(current_child as usize).unwrap().symbol).unwrap()
//                         )
//                         .as_str(),
//                     );
//                     info!("{}", b);
//                     b.clear();

//                     // Go deeper or process current node.
//                     if !current.children.get(current_child as usize).unwrap().children.is_empty() {
//                         // Push onto stack
//                         stack.push(VecDeque::new());

//                         let child = current.children.remove(current_child as usize);
//                         current_child -= 1;
//                         let len = (child.children.len() - 1) as i32;
//                         node_stack.push(current);
//                         node_stack.push(child);
//                         child_count_stack.push((current_child, min_child));
//                         child_count_stack.push((len, 0));
//                         break;
//                     } else {
//                         let child = current.children.get(current_child as usize).unwrap().clone();
//                         let wrong_data = || panic!("I'm too tired to write this error message properly.");
//                         if let Some(last) = stack.last_mut() {
//                             if let Some(data) = child.data {
//                                 match data {
//                                     FernData::Number(n) => {
//                                         if child.symbol == tok.number {
//                                             last.push_back(AstNode::Number(n));
//                                         } else {
//                                             wrong_data();
//                                         }
//                                     }
//                                     FernData::String(s) => {
//                                         if child.symbol == tok.name {
//                                             last.push_back(AstNode::Name(s));
//                                         } else if child.symbol == tok.string {
//                                             last.push_back(AstNode::String(s));
//                                         } else {
//                                             wrong_data();
//                                         }
//                                     }
//                                     FernData::NoData => (),
//                                 }
//                             }
//                         }
//                     }
//                     current_child -= 1;
//                     if current_child < min_child {
//                         if let Some(root) = new_reduce(current, &mut stack, &tok, &self.g) {
//                             return Ok(root);
//                         }
//                         break;
//                     }
//                 }
//             } else {
//                 if let Some(root) = new_reduce(current, &mut stack, &tok, &self.g) {
//                     return Ok(root);
//                 }
//             }
//         }
//         Err(SimpleError::new("Failed to build full ast from parse tree."))
//     }
// }

// impl Debug for Operator {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         match self {
//             Add => write!(f, "+"),
//             Multiply => write!(f, "*"),
//             Divide => write!(f, "/"),
//             Subtract => write!(f, "-"),
//             GreaterThan => write!(f, "gt"),
//             LessThan => write!(f, "lt"),
//             Modulo => write!(f, "%"),
//             Equal => write!(f, "=="),
//             NotEqual => write!(f, "!="),
//             GreaterThanOrEqual => write!(f, "gt="),
//             LessThanOrEqual => write!(f, "lt="),
//         }
//     }
// }

// impl Debug for AstNode {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         match self {
//             AstNode::Binary(_l, o, _r) => write!(f, "Binary<BR/>{:?}", o),
//             AstNode::Unary(o, _e) => write!(f, "Unary<BR/>{:?}", o),
//             AstNode::Number(n) => write!(f, "{}", n),
//             AstNode::String(s) => {
//                 write!(f, "\"{}\"", s)
//             }
//             AstNode::FunctionCall(_, _) => write!(f, "Function Call"),
//             AstNode::Name(n) => write!(f, "{}", n),
//             AstNode::ExprList(_) => write!(f, "Expr List"),
//             AstNode::Assign(_, _) => write!(f, "="),
//             AstNode::Let(_, _, _) => write!(f, "Let"),
//             AstNode::Module(_) => write!(f, "Module"),
//             AstNode::Function(name, _, _) => write!(f, "Function<BR/>{:?}", name),
//             AstNode::If(_, _, _) => write!(f, "If"),
//             AstNode::ExprThen(_, _) => write!(f, "Expr Then"),
//             AstNode::ElseIf(_, _, _) => write!(f, "Else If"),
//             AstNode::Else(_) => write!(f, "Else"),
//             AstNode::For(_, _, _) => write!(f, "For"),
//             AstNode::While(_, _) => write!(f, "While"),
//             AstNode::Return(_) => write!(f, "Return"),
//             AstNode::StatList(_) => write!(f, "Statement List"),
//         }
//     }
// }

type Nd = (usize, String);
type Ed = (Nd, Nd);
struct Graph {
    nodes: Vec<String>,
    edges: VecDeque<(usize, usize)>,
}

impl<'a> dot::Labeller<'a, Nd, Ed> for Graph {
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new("example3").unwrap()
    }
    fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
        dot::Id::new(format!("N{}", n.0)).unwrap()
    }
    fn node_label(&self, n: &Nd) -> dot::LabelText {
        let &(i, _) = n;
        dot::LabelText::HtmlStr(self.nodes[i].clone().into())
    }
    fn edge_label(&self, _: &Ed) -> dot::LabelText {
        dot::LabelText::LabelStr("".into())
    }
}

impl<'a> dot::GraphWalk<'a, Nd, Ed> for Graph {
    fn nodes(&'a self) -> dot::Nodes<'a, Nd> {
        let new_nodes = self.nodes.clone().into_iter().enumerate().collect();
        Cow::Owned(new_nodes)
    }
    fn edges(&'a self) -> dot::Edges<'a, Ed> {
        self.edges
            .iter()
            .map(|&(i, j)| ((i, self.nodes[i].clone()), (j, self.nodes[j].clone())))
            .collect()
    }
    fn source(&self, e: &Ed) -> Nd {
        e.0.clone()
    }
    fn target(&self, e: &Ed) -> Nd {
        e.1.clone()
    }
}

// pub fn render<W: Write>(ast: Box<AstNode>, output: &mut W) {
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
