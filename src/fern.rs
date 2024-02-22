use crate::grammar::lg::{self, LexingTable, LookupResult, State};
use crate::grammar::opg::{OpGrammar, RawGrammar, Token};
use crate::lexer::{split_mmap_into_chunks, Data, LexerError, LexerInterface, ParallelLexer};
use crate::parser::{Parser, PartialParseTree};
use crate::parsetree::{Node, ParseTree};
use log::{info, trace, warn};
use memmap::MmapOptions;
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
        let chunks = split_mmap_into_chunks(&mut mmap, 1000).unwrap();
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
    grammar.to_file("data/grammar/fern-fnf.g");

    let parse_time = Instant::now();
    let tree: ParseTree = {
        let mut trees = Vec::new();
        for (partial_tokens, partial_data) in tokens {
            let mut parser = Parser::new(grammar.clone());
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

    tree.print();
    let mut f = File::create("ptree.dot").unwrap();
    tree.dot(&mut f).unwrap();

    // let ast: FernAst = tree.into();
    // ast.print();
    // let mut f = File::create("ast.dot").unwrap();
    // ast.dot(&mut f).unwrap();

    info!("Time to build first lexical grammar: {:?}", first_lg);
    info!("Time to build second lexical grammar: {:?}", second_lg);
    info!("Time to lex: {:?}", lex_time);
    info!("Time to build parsing grammar: {:?}", grammar_time);
    info!("Time to parse: {:?}", parse_time);
    // info!("└─Time spent rule-searching: {:?}", time);
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
                }
            }
        }
    }
}

#[derive(Clone)]
pub enum Operator {
    Add,
    Multiply,
    Divide,
    Modulo,
    Subtract,
    Equal,
    NotEqual,
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AstNodeKind {
    Binary,
    Unary,
    Number,
    String,
    Name,
    ExprList,
    Assign,
    Let,
    Return,
    Module,
    StatList,
    FunctionCall,
    Function,
    If,
    ExprThen,
    ElseIf,
    Else,
    For,
    While,
}

struct FernAst {
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
        let string = find("STRING");
        let name = find("NAME");
        let number = find("NUMBER");
        let let_t = find("LET");
        let eq = find("EQ");
        let semi = find("SEMI");
        let stat = find("stat");
        let stat_list = find("statList");
        let axiom = find("NewAxiom");
        let mut reduce = |parent: &Node, children: Vec<Node>| -> AstNode {
            if base_exp.contains(&parent.token) {
                let child = children.get(0).unwrap().token;
                if string.contains(&child) {
                    return AstNode {
                        kind: AstNodeKind::String,
                        child_count: 0,
                    };
                } else if name.contains(&child) {
                    return AstNode {
                        kind: AstNodeKind::Name,
                        child_count: 0,
                    };
                } else if number.contains(&child) {
                    return AstNode {
                        kind: AstNodeKind::Number,
                        child_count: 0,
                    };
                }
            }

            if stat.contains(&parent.token) {
                if let_t.contains(&children.get(0).unwrap().token) {
                    return AstNode {
                        kind: AstNodeKind::Let,
                        child_count: 2,
                    };
                }
            }

            if stat_list.contains(&parent.token) {
                let mut cnt = 0;
                for c in children {
                    if !semi.contains(&c.token) {
                        cnt += 1;
                    }
                }
                return AstNode {
                    kind: AstNodeKind::StatList,
                    child_count: cnt,
                };
            }

            todo!();
        };

        let mut nodes = Vec::new();
        let mut current = Vec::new();
        let mut operands: Vec<Node> = Vec::new();
        for n in self.nodes {
            let ops: Vec<&String> = operands.iter().map(|n| self.token_map.get(&n.token).unwrap()).collect();
            info!("operands {:?}", ops);
            if n.child_count > 0 {
                let ops: Vec<Node> = operands.drain(operands.len() - n.child_count..).collect();
                let ops_str: Vec<&String> = ops.iter().map(|n| self.token_map.get(&n.token).unwrap()).collect();
                info!("reduce {:?}: {:?} ", self.token_map.get(&n.token).unwrap(), ops_str);

                if axiom.contains(&n.token) {
                    break;
                }
                let new = reduce(&n, ops);
                if new.kind == AstNodeKind::StatList {}
                nodes.push(new);
                current.push(nodes.len() - 1);
                operands.push(n);
            } else {
                operands.push(n);
            }
        }
        let n: Vec<(AstNodeKind, usize)> = nodes.iter().map(|n| (n.kind, n.child_count)).collect();
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
                edges.push_back((stack.last().unwrap().0.unwrap(), current));
            }
        });
        let g = Graph { nodes, edges };
        dot::render(&g, out)
    }

    pub fn print(&self) {
        self.pre_order_traverse(|stack, current| {
            let n = &self.nodes[current];

            let mut padding = String::new();
            for i in 0..stack.len() - 1 {
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
