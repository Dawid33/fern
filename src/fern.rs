use crate::grammar::lg::{self, LexingTable, LookupResult, State};
use crate::grammar::opg::{OpGrammar, RawGrammar, Token};
use crate::lexer::{Data, LexerError, LexerInterface, ParallelLexer};
use crate::parser::{Parser, PartialParseTree};
use crate::parsetree::{Node, ParseTree};
use log::{info, trace, warn};
use simple_error::SimpleError;
use std::borrow::Cow;
use std::cmp::max;
use std::collections::{BTreeMap, HashMap, LinkedList, VecDeque};
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::{self, Read, Write};
use std::time::{Duration, Instant};
use std::{sync, thread};

pub struct FernParseTree {
    pub g: OpGrammar,
    pub root: Node,
}

#[cfg(not(target_arch = "wasm32"))]
pub fn compile() -> Result<(), Box<dyn Error>> {
    use memmap::MmapOptions;

    use crate::split_file_into_chunks;

    let start = Instant::now();

    let first_lg = Instant::now();
    let mut file = File::open("data/grammar/fern.lg").unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    let g = lg::LexicalGrammar::from(&buf);
    let nfa = lg::StateGraph::from(g.clone());
    let mut f = File::create("nfa.dot").unwrap();
    lg::render(&nfa, &mut f);
    let dfa = nfa.convert_to_dfa();
    let mut f = File::create("dfa.dot").unwrap();
    lg::render(&dfa, &mut f);
    let mut table = dfa.build_table();
    table.terminal_map.push("UMINUS".to_string());
    let first_lg = first_lg.elapsed();
    buf.clear();

    let second_lg = Instant::now();
    let mut file = File::open("data/grammar/keywords.lg").unwrap();
    file.read_to_string(&mut buf).unwrap();
    let g = lg::LexicalGrammar::from(&buf);
    let nfa = lg::StateGraph::from(g.clone());
    let mut f = File::create("keyword_nfa.dot").unwrap();
    lg::render(&nfa, &mut f);
    let dfa = nfa.convert_to_dfa();
    let mut f = File::create("keyword_dfa.dot").unwrap();
    lg::render(&dfa, &mut f);
    let keywords = dfa.build_table();
    let second_lg = second_lg.elapsed();

    let name_token = table.terminal_map.iter().position(|x| x == "NAME").unwrap();
    table.add_table(name_token, keywords);

    let lex_time = Instant::now();
    let tokens: LinkedList<(Vec<Token>, Vec<Data>)> = {
        let file = File::open("data/test.fern")?;
        let mut mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };
        let chunks = split_file_into_chunks(&mmap, 1000).unwrap();
        thread::scope(|s| {
            let mut lexer: ParallelLexer<FernLexer> = ParallelLexer::new(table.clone(), s, 1);
            let batch = lexer.new_batch();
            for task in chunks.iter().enumerate() {
                lexer.add_to_batch(&batch, task.1, task.0);
            }
            let tokens = lexer.collect_batch(batch);
            lexer.kill();
            tokens
        })
    };
    let lex_time = lex_time.elapsed();

    // info!("{:?}", table.terminal_map);
    // info!("{:?}", &tokens);
    // for (l, _) in &tokens {
    //     for t in l {
    //         info!("{}", table.terminal_map[*t]);
    //     }
    // }

    let grammar_time = Instant::now();
    let mut raw = RawGrammar::from("data/grammar/fern.g", table.terminal_map.clone()).unwrap();
    raw.delete_repeated_rhs()?;
    let grammar = OpGrammar::new(raw)?;
    let grammar_time = grammar_time.elapsed();

    let parse_time = Instant::now();
    let tree: ParseTree = {
        let mut trees = Vec::new();
        for (partial_tokens, partial_data) in tokens {
            let mut parser = Parser::new(grammar.clone());
            for (i, (x, y)) in partial_tokens.iter().zip(&partial_data).enumerate() {
                info!("i={}, {} {:?}", i, x, y);
            }
            parser.parse(partial_tokens, partial_data);
            parser.parse(vec![grammar.delim], Vec::new());
            trees.push(parser.collect_parse_tree().unwrap());
        }

        trees.reverse();
        let mut first = trees.pop().unwrap();
        while let Some(tree) = trees.pop() {
            first.merge(tree);
        }
        first.into_tree()
    };
    let parse_time = parse_time.elapsed();

    // tree.print_raw();
    tree.print();
    let mut f = File::create("ptree.dot").unwrap();
    tree.dot(&mut f).unwrap();

    let ast: FernAst = tree.into();
    ast.print();
    let mut f = File::create("ast.dot").unwrap();
    ast.dot(&mut f).unwrap();

    info!("Time to build first lexical grammar: {:?}", first_lg);
    info!("Time to build second lexical grammar: {:?}", second_lg);
    info!("Time to lex: {:?}", lex_time);
    info!("Time to build parsing grammar: {:?}", grammar_time);
    info!("Time to parse: {:?}", parse_time);
    // info!("└─Time spent rule-searching: {:?}", time);
    info!("Total run time : {:?}", start.elapsed());

    for s in ast.analysis() {
        warn!("{}", s);
    }
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
    semi: Token,
    lbrace: Token,
    rbrace: Token,
    let_t: Token,
    while_t: Token,
    return_t: Token,
    fn_t: Token,
    minus: Token,
    unary_minus: Token,
    name_token: Token,
    lparen: Token,
    rparen: Token,
    comment: Token,
}

impl LexerInterface for FernLexer {
    fn new(table: LexingTable, start_state: usize) -> Self {
        let name_token = table.terminal_map.iter().position(|x| x == "NAME").unwrap();
        let lparen = table.terminal_map.iter().position(|x| x == "LPAREN").unwrap();
        let rparen = table.terminal_map.iter().position(|x| x == "RPAREN").unwrap();
        let whitespace_token = table.terminal_map.iter().position(|x| x == "WHITESPACE").unwrap();
        let minus = table.terminal_map.iter().position(|x| x == "MINUS").unwrap();
        let unary_minus = table.terminal_map.iter().position(|x| x == "UMINUS").unwrap();
        let fn_t = table.terminal_map.iter().position(|x| x == "FUNCTION").unwrap();
        let return_t = table.terminal_map.iter().position(|x| x == "RETURN").unwrap();
        let while_t = table.terminal_map.iter().position(|x| x == "WHILE").unwrap();
        let let_t = table.terminal_map.iter().position(|x| x == "LET").unwrap();
        let rbrace = table.terminal_map.iter().position(|x| x == "RBRACE").unwrap();
        let lbrace = table.terminal_map.iter().position(|x| x == "LBRACE").unwrap();
        let semi = table.terminal_map.iter().position(|x| x == "SEMI").unwrap();
        let comment = table.terminal_map.iter().position(|x| x == "COMMENT").unwrap();

        Self {
            table,
            whitespace_token,
            unary_minus,
            minus,
            had_whitespace: false,
            had_lparenfunc: -1,
            name_token,
            lparen,
            rparen,
            tokens: Vec::new(),
            start_state,
            buf: String::new(),
            state: start_state,
            data: Vec::new(),
            semi,
            lbrace,
            rbrace,
            let_t,
            while_t,
            return_t,
            fn_t,
            comment,
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

                    trace!("c, t: {}, {}", input as char, self.table.terminal_map[t]);
                    if t != self.whitespace_token && t != self.comment {
                        let mut t2 = if self.had_whitespace { self.whitespace_token } else { t };
                        self.look_ahead(&mut t2);
                        self.look_ahead_no_whitespace(&mut t);
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
                    trace!("Lexing Error when transitioning state. state : {}", self.state);
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
    fn look_ahead(&mut self, t2: &mut Token) {
        if let Some(t1) = self.tokens.last() {
            trace!("look_ahead {}, {}", self.table.terminal_map[*t1], self.table.terminal_map[*t2]);
            if *t1 == self.minus && (*t2 == self.name_token || *t2 == self.lparen) {
                *self.tokens.last_mut().unwrap() = self.unary_minus;
            }
        }
    }
    fn look_ahead_no_whitespace(&mut self, t2: &mut Token) {
        if let Some(t1) = self.tokens.last() {
            if *t1 == self.rbrace {
                if *t2 == self.let_t || *t2 == self.name_token || *t2 == self.return_t || *t2 == self.fn_t || *t2 == self.rbrace {
                    self.tokens.push(self.semi);
                    self.data.push(Data {
                        token_index: self.tokens.len() - 1,
                        raw: ";".to_string(),
                    });
                }
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OperatorKind {
    Add,
    Multiply,
    Divide,
    Modulo,
    Subtract,
    Equal,
    NotEqual,
    Or,
    And,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

#[derive(Debug)]
struct AstNode {
    kind: AstNodeKind,
    child_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AstNodeKind {
    Operator(OperatorKind),
    Number(String),
    String(String),
    Name(String),
    Field,
    ExprList,
    FieldList,
    Assign,
    Let,
    LetAssign,
    Return,
    Module,
    StatList,
    FunctionCall,
    Function,
    If,
    ElseIf,
    Else,
    For,
    While,
    Struct,
}

pub struct FernAst {
    nodes: Vec<AstNode>,
    token_map: BTreeMap<usize, String>,
}

impl Into<FernAst> for ParseTree {
    fn into(self) -> FernAst {
        let find = |tok: &str| -> Vec<usize> {
            let mut res = Vec::new();
            for (k, v) in &self.token_map {
                if v == tok {
                    res.push(*k);
                }
            }
            return res;
        };

        let base_exp = find("baseExp");
        let prefix_exp = find("prefixExp");
        let string = find("STRING");
        let else_t = find("ELSE");
        let else_if = find("ELSEIF");
        let rbrack = find("RBRACK");
        let rbrace = find("RBRACE");
        let name = find("NAME");
        let number = find("NUMBER");
        let let_t = find("LET");
        let if_t = find("IF");
        let fn_t = find("FUNCTION");
        let eq = find("EQ");
        let colon = find("COLON");
        let or = find("logicalOrExp");
        let and = find("logicalAndExp");
        let semi = find("SEMI");
        let stat = find("stat");
        let field = find("field");
        let stat_list = find("statList");
        let field_list = find("fieldList");
        let field_list_body = find("fieldListBody");
        let axiom = find("NewAxiom");
        let relational_exp = find("relationalExp");
        let additive_exp = find("additiveExp");
        let mul_exp = find("multiplicativeExp");
        let ret_stat = find("retStat");
        let fn_call = find("functionCall");
        let expr_then = find("exprThen");
        let lbrace = find("LBRACE");
        let struct_t = find("STRUCT");
        let else_if_block = find("elseIfBlock");

        let rm_child = |mut child: isize, mut total_child: isize, existing_nodes: &mut Vec<AstNode>| -> Vec<AstNode> {
            let desired = child;
            let mut current_child = 0;
            let mut seeking_end = false;
            let mut start_index = 0;
            let mut end_index = 0;
            total_child -= 1;
            for (i, n) in existing_nodes.iter().enumerate().rev() {
                warn!("i: {}, child {}, total: {}, node: {:?}", i, child, total_child, n);
                child -= 1;
                child += n.child_count as isize;
                if child - (total_child - 1) == 0 {
                    total_child -= 1;
                    current_child += 1;
                    if !seeking_end {
                        if current_child == desired {
                            warn!("END i: {}, child {}, total: {}, node: {:?}", i, child, total_child, n);
                            end_index = i;
                            seeking_end = true;
                        }
                    } else {
                        warn!("START i: {}, child {}, total: {}, node: {:?}", i, child, total_child, n);
                        start_index = i;
                        break;
                    }
                }
                if child < 0 {
                    warn!("START i: {}, child {}, total: {}, node: {:?}", i, child, total_child, n);
                    start_index = i;
                    break;
                };
            }
            warn!("start: {}, end {}", start_index, end_index);

            if start_index == end_index {
                vec![existing_nodes.remove(start_index)]
            } else {
                existing_nodes.drain(start_index..end_index).collect()
            }
        };

        let expr_map = |parent: &Node, children: &Vec<Node>| -> Option<AstNode> {
            if base_exp.contains(&parent.token) {
                let child = children.last().unwrap().token;
                let data = if let Some(ref d) = children.get(0).unwrap().data {
                    d.raw.clone()
                } else {
                    String::new()
                };
                if string.contains(&child) {
                    return Some(AstNode {
                        kind: AstNodeKind::String(data),
                        child_count: 0,
                    });
                } else if name.contains(&child) {
                    return Some(AstNode {
                        kind: AstNodeKind::Name(data),
                        child_count: 0,
                    });
                } else if number.contains(&child) {
                    return Some(AstNode {
                        kind: AstNodeKind::Number(data),
                        child_count: 0,
                    });
                }
            }

            if relational_exp.contains(&parent.token) {
                return Some(AstNode {
                    kind: AstNodeKind::Operator(OperatorKind::Equal),
                    child_count: 2,
                });
            }

            if additive_exp.contains(&parent.token) {
                return Some(AstNode {
                    kind: AstNodeKind::Operator(OperatorKind::Add),
                    child_count: 2,
                });
            }

            if mul_exp.contains(&parent.token) {
                return Some(AstNode {
                    kind: AstNodeKind::Operator(OperatorKind::Multiply),
                    child_count: 2,
                });
            }

            if or.contains(&parent.token) {
                return Some(AstNode {
                    kind: AstNodeKind::Operator(OperatorKind::Or),
                    child_count: 2,
                });
            }

            if and.contains(&parent.token) {
                return Some(AstNode {
                    kind: AstNodeKind::Operator(OperatorKind::And),
                    child_count: 2,
                });
            }
            None
        };

        let reduce = |parent: &Node, mut children: Vec<Node>, existing_nodes: &mut Vec<AstNode>| -> Option<AstNode> {
            if let Some(op) = expr_map(parent, &children) {
                return Some(op);
            }

            if prefix_exp.contains(&parent.token) {
                warn!("does nothing for prefixexp");
                return None;
            }

            if ret_stat.contains(&parent.token) {
                return Some(AstNode {
                    kind: AstNodeKind::Return,
                    child_count: 1,
                });
            }

            if fn_call.contains(&parent.token) {
                if children.len() > 3 {
                    return Some(AstNode {
                        kind: AstNodeKind::FunctionCall,
                        child_count: 2,
                    });
                } else {
                    return Some(AstNode {
                        kind: AstNodeKind::FunctionCall,
                        child_count: 1,
                    });
                }
            }

            if field.contains(&parent.token) {
                return Some(AstNode {
                    kind: AstNodeKind::Field,
                    child_count: 2,
                });
            }

            if field_list.contains(&parent.token) {
                return Some(AstNode {
                    kind: AstNodeKind::FieldList,
                    child_count: 1,
                });
            }

            if field_list_body.contains(&parent.token) {
                if children.len() > 1 {
                    return Some(AstNode {
                        kind: AstNodeKind::FieldList,
                        child_count: 2,
                    });
                }
            }

            if else_if_block.contains(&parent.token) {
                if else_t.contains(&children.last().unwrap().token) {
                    if children.len() > 3 {
                        return Some(AstNode {
                            kind: AstNodeKind::Else,
                            child_count: 1,
                        });
                    } else {
                        return Some(AstNode {
                            kind: AstNodeKind::Else,
                            child_count: 0,
                        });
                    }
                }
                if else_if.contains(&children.last().unwrap().token) {
                    if children.len() == 4 {
                        return Some(AstNode {
                            kind: AstNodeKind::ElseIf,
                            child_count: 1,
                        });
                    } else if children.len() == 5 {
                        return Some(AstNode {
                            kind: AstNodeKind::ElseIf,
                            child_count: 2,
                        });
                    } else if children.len() == 6 {
                        return Some(AstNode {
                            kind: AstNodeKind::ElseIf,
                            child_count: 3,
                        });
                    }
                }
            }

            if stat.contains(&parent.token) {
                if struct_t.contains(&children.last().unwrap().token) {
                    if children.len() > 3 {
                        return Some(AstNode {
                            kind: AstNodeKind::Struct,
                            child_count: 2,
                        });
                    } else {
                        return Some(AstNode {
                            kind: AstNodeKind::Struct,
                            child_count: 1,
                        });
                    }
                }

                if lbrace.contains(&children.last().unwrap().token) {
                    return Some(AstNode {
                        kind: AstNodeKind::StatList,
                        child_count: 0,
                    });
                }

                if eq.contains(&children.last().unwrap().token) {
                    return Some(AstNode {
                        kind: AstNodeKind::Assign,
                        child_count: 2,
                    });
                }

                if fn_t.contains(&children.last().unwrap().token) {
                    if children.len() == 6 {
                        return Some(AstNode {
                            kind: AstNodeKind::Function,
                            child_count: 1,
                        });
                    } else if rbrack.contains(&children.get(children.len() - 4).unwrap().token) {
                        return Some(AstNode {
                            kind: AstNodeKind::Function,
                            child_count: 2,
                        });
                    } else if !rbrace.contains(&children.get(1).unwrap().token) {
                        return Some(AstNode {
                            kind: AstNodeKind::Function,
                            child_count: 3,
                        });
                    } else {
                        return Some(AstNode {
                            kind: AstNodeKind::Function,
                            child_count: 2,
                        });
                    }
                }

                if if_t.contains(&children.last().unwrap().token) {
                    let mut cnt = 1;
                    if else_if_block.contains(&children.last().unwrap().token) {
                        cnt += 1;
                    }
                    if !rbrace.contains(&children.get(3).unwrap().token) {
                        cnt += 1;
                    }
                    return Some(AstNode {
                        kind: AstNodeKind::If,
                        child_count: cnt,
                    });
                }

                if let_t.contains(&children.last().unwrap().token) {
                    if children.len() > 2 {
                        return Some(AstNode {
                            kind: AstNodeKind::LetAssign,
                            child_count: 2,
                        });
                    } else {
                        return Some(AstNode {
                            kind: AstNodeKind::Let,
                            child_count: 1,
                        });
                    }
                }
            }

            if stat_list.contains(&parent.token) {
                let mut cnt = 0;
                for c in children {
                    if !semi.contains(&c.token) {
                        cnt += 1;
                    }
                }
                return Some(AstNode {
                    kind: AstNodeKind::StatList,
                    child_count: cnt,
                });
            }

            todo!("Didn't reduce parse tree node.");
        };

        let mut nodes: Vec<AstNode> = Vec::new();
        // let mut current = Vec::new();
        let mut operands: Vec<Node> = Vec::new();
        // bottom up traversal of the tree.
        for n in self.nodes.into_iter().rev() {
            let ops: Vec<&String> = operands.iter().map(|n| self.token_map.get(&n.token).unwrap()).collect();
            info!("operands {:?}", ops);
            if n.child_count > 0 {
                let ops: Vec<Node> = operands.drain(operands.len() - n.child_count..).collect();
                let ops_str: Vec<&String> = ops.iter().map(|n| self.token_map.get(&n.token).unwrap()).collect();
                info!("reduce {:?}: {:?} ", self.token_map.get(&n.token).unwrap(), ops_str);

                if axiom.contains(&n.token) {
                    break;
                }
                let mut new = match reduce(&n, ops.clone(), &mut nodes) {
                    Some(new) => new,
                    None => {
                        operands.push(n);
                        continue;
                    }
                };

                // Flatten statlists
                if new.kind == AstNodeKind::StatList {
                    let mut has_child_stat_list = false;
                    for o in &ops {
                        if stat_list.contains(&o.token) {
                            has_child_stat_list = true;
                            break;
                        }
                    }
                    if has_child_stat_list {
                        // Go through the exsiting ast node stack backwards
                        // in order to find the index of the existing statlist.
                        let mut stat_list_pos = None;
                        for (i, x) in nodes.iter().enumerate().rev() {
                            if x.kind == AstNodeKind::StatList {
                                stat_list_pos = Some(i);
                                break;
                            }
                        }
                        if let Some(i) = stat_list_pos {
                            let mut existing_stat_list = nodes.remove(i);
                            existing_stat_list.child_count += new.child_count - 1;
                            new = existing_stat_list;
                        }
                    }
                }

                nodes.push(new);
                operands.push(n);
            } else {
                operands.push(n);
            }
        }
        let n: Vec<(AstNodeKind, usize)> = nodes.iter().map(|n| (n.kind.clone(), n.child_count)).collect();
        info!("ast {:?}", n);
        FernAst {
            nodes,
            token_map: self.token_map,
        }
    }
}

impl FernAst {
    fn pre_order_traverse<F: FnMut(&Vec<(Option<usize>, usize)>, usize)>(&self, mut f: F) {
        let mut stack = Vec::from(&[(None, self.nodes.last().unwrap().child_count)]);

        for (i, n) in self.nodes.iter().enumerate().rev() {
            let last = stack.last_mut().unwrap();
            last.1 -= 1;

            f(&stack, i);

            if n.child_count > 0 {
                stack.push((Some(i), n.child_count as usize));
            } else {
                while let Some((_, last)) = stack.last() {
                    if *last == 0 {
                        stack.pop();
                    } else {
                        break;
                    }
                }
            }
        }
    }

    pub fn dot<W: Write>(&self, out: &mut W) -> io::Result<()> {
        let nodes = self.nodes.iter().map(|n| format!("{:?}", n.kind)).collect();
        let mut edges = VecDeque::new();

        self.pre_order_traverse(|stack, current| {
            if stack.last().unwrap().0.is_some() {
                edges.push_front((stack.last().unwrap().0.unwrap(), current));
            }
        });
        let g = Graph { nodes, edges };
        dot::render(&g, out)
    }

    pub fn print(&self) {
        info!("{:?}", self.nodes.last().unwrap().kind);
        self.pre_order_traverse(|stack, current| {
            // don't print first node during traversal
            if current == self.nodes.len() - 1 {
                return;
            }

            let n = &self.nodes[current];

            let mut padding = String::new();
            for i in 1..stack.len() - 1 {
                let current = stack.get(i).unwrap();
                if current.1 > 0 {
                    padding.push_str("| ");
                } else {
                    padding.push_str("  ");
                }
            }

            let last = stack.last().unwrap();
            if last.1 > 0 {
                info!("{}├─{:?}", padding, n.kind);
            } else {
                info!("{}└─{:?}", padding, n.kind);
            }
        });
    }

    pub fn analysis(&self) -> Vec<String> {
        let mut table: BTreeMap<String, IdentifierKind> = BTreeMap::new();
        let mut prefix: Vec<String> = Vec::new();
        let mut partial_var = None;
        let mut issues_discovered = Vec::new();
        self.pre_order_traverse(|stack, current| {
            let n = &self.nodes[current];
            let (parent_id, children_left) = if let Some((p_id, child_count)) = stack.last() {
                (p_id, child_count)
            } else {
                panic!("One name node with no parent??");
            };

            let mut add_to_table = |name: &String, data: IdentifierKind, table: &mut BTreeMap<String, IdentifierKind>| {
                if table.contains_key(name) {
                    issues_discovered.push(format!("Identifier {} already exists.", name));
                } else {
                    table.insert(name.clone(), data);
                }
            };

            match n.kind {
                AstNodeKind::Number(ref name) | AstNodeKind::String(ref name) => match self.nodes[parent_id.unwrap()].kind {
                    AstNodeKind::LetAssign => {
                        if !name.is_empty() {
                            warn!("{}", name);
                            if *children_left == 1 {
                                issues_discovered.push(format!("Strings and / or numbers ({}) cannot be used as identifiers.", name));
                            } else if *children_left == 0 {
                                if let Some(left) = partial_var {
                                    partial_var = None;
                                    add_to_table(left, IdentifierKind::Local, &mut table);
                                }
                            }
                        }
                    }
                    _ => (),
                },
                AstNodeKind::Operator(_) | AstNodeKind::FunctionCall => match self.nodes[parent_id.unwrap()].kind {
                    AstNodeKind::LetAssign => {
                        if let Some(left) = partial_var {
                            partial_var = None;
                            add_to_table(left, IdentifierKind::Local, &mut table);
                        }
                    }
                    _ => (),
                },
                AstNodeKind::Name(ref name) => match self.nodes[parent_id.unwrap()].kind {
                    AstNodeKind::LetAssign => {
                        if !name.is_empty() {
                            warn!("{}", name);
                            if *children_left == 1 {
                                partial_var = Some(name);
                            } else if *children_left == 0 {
                                if let Some(left) = partial_var {
                                    partial_var = None;
                                    if !table.contains_key(name) {
                                        issues_discovered.push(format!("Identifier {} used but not declared.", name));
                                    } else {
                                        add_to_table(left, IdentifierKind::Local, &mut table);
                                    }
                                }
                            }
                        }
                    }
                    AstNodeKind::Let => {
                        if !name.is_empty() {
                            add_to_table(name, IdentifierKind::Local, &mut table);
                        }
                    }
                    AstNodeKind::Function => match n.kind {
                        AstNodeKind::String(ref name) | AstNodeKind::Number(ref name) => {
                            issues_discovered.push(format!("Invalid function name {}", name));
                        }
                        AstNodeKind::Name(ref name) => {
                            warn!("{}", name);
                            add_to_table(name, IdentifierKind::FunctionName, &mut table);
                        }
                        _ => {}
                    },
                    AstNodeKind::Field => {
                        if !name.is_empty() && *children_left == 1 {
                            partial_var = Some(name);
                        } else if *children_left == 0 {
                            let n = partial_var.unwrap();
                            partial_var = None;
                            add_to_table(n, IdentifierKind::FunctionParam(name.clone()), &mut table);
                        }
                    }
                    _ => match n.kind {
                        AstNodeKind::Name(ref name) => {
                            if !table.contains_key(name) {
                                issues_discovered.push(format!("Identifier {} used but not declared.", name));
                            }
                        }
                        _ => (),
                    },
                    _ => (),
                },
                _ => (),
            }
        });
        warn!("SBL_TBL: {:?}", table);
        issues_discovered
    }
}

#[derive(Debug)]
enum IdentifierKind {
    FunctionName,
    FunctionParam(String),
    Local,
}

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
