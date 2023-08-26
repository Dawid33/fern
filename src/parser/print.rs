use crate::grammar::{Associativity, OpGrammar, Rule, Token};
use crate::ir::{Block, BlockType, Statement};
use crate::lexer::fern::FernData;
use crate::parser::fern_ast::Operator::{Add, Divide, Equal, GreaterThan, GreaterThanOrEqual, LessThan, LessThanOrEqual, Modulo, Multiply, NotEqual, Subtract};
use crate::parser::{Node, ParseTree};
use log::info;
use log::{debug, error, warn};
use simple_error::SimpleError;
use std::any::Any;
use std::borrow::Cow;
use std::cmp::max;
use std::collections::{BTreeSet, HashMap, LinkedList, VecDeque};
use std::error::Error;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::hint::unreachable_unchecked;
use std::io::ErrorKind::AlreadyExists;
use std::io::Read;
use std::io::Write;
use std::marker::PhantomData;
use std::panic::{resume_unwind, set_hook};
use std::slice::Iter;
use std::sync;
use std::sync::mpsc::channel;
use std::thread::current;
use std::time::Duration;
// use tokio::time::Instant;

use super::fern_ast::{AstNode, FernParseTree, Operator};

impl Debug for Operator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Add => write!(f, "+"),
            Multiply => write!(f, "*"),
            Divide => write!(f, "/"),
            Subtract => write!(f, "-"),
            GreaterThan => write!(f, "gt"),
            LessThan => write!(f, "lt"),
            Modulo => write!(f, "%"),
            Equal => write!(f, "=="),
            NotEqual => write!(f, "!="),
            GreaterThanOrEqual => write!(f, "gt="),
            LessThanOrEqual => write!(f, "lt="),
        }
    }
}

impl Debug for AstNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AstNode::Binary(_l, o, _r) => write!(f, "Binary<BR/>{:?}", o),
            AstNode::Unary(o, _e) => write!(f, "Unary<BR/>{:?}", o),
            AstNode::Number(n) => write!(f, "{}", n),
            AstNode::String(s) => {
                write!(f, "\"{}\"", s)
            }
            AstNode::FunctionCall(_, _) => write!(f, "Function Call"),
            AstNode::Name(n) => write!(f, "{}", n),
            AstNode::ExprList(_) => write!(f, "Expr List"),
            AstNode::Assign(_, _) => write!(f, "="),
            AstNode::Let(_, _, _) => write!(f, "Let"),
            AstNode::Module(_) => write!(f, "Module"),
            AstNode::Function(name, _, _) => write!(f, "Function<BR/>{:?}", name),
            AstNode::If(_, _, _) => write!(f, "If"),
            AstNode::ExprThen(_, _) => write!(f, "Expr Then"),
            AstNode::ElseIf(_, _, _) => write!(f, "Else If"),
            AstNode::Else(_) => write!(f, "Else"),
            AstNode::For(_, _, _) => write!(f, "For"),
            AstNode::While(_, _) => write!(f, "While"),
            AstNode::Return(_) => write!(f, "Return"),
            AstNode::StatList(_) => write!(f, "Statement List"),
        }
    }
}

impl ParseTree<FernData> for FernParseTree {
    fn new(root: Node<FernData>, g: OpGrammar) -> Self {
        Self { g, root }
    }

    fn print(&self) {
        let mut node_stack: Vec<&Node<FernData>> = vec![&self.root];
        let mut child_count_stack: Vec<(i32, i32)> = vec![((self.root.children.len() - 1) as i32, 0)];
        let mut b = String::new();

        b.push_str(format!("{}", self.g.token_raw.get(&self.root.symbol).unwrap()).as_str());
        info!("{}", b);
        b.clear();
        while !node_stack.is_empty() {
            let current = node_stack.pop().unwrap();
            let (mut current_child, min_child) = child_count_stack.pop().unwrap();

            while current.children.len() > 0 && current_child >= min_child {
                for i in 0..child_count_stack.len() {
                    let (current, min) = child_count_stack.get(i).unwrap();
                    if *current >= *min {
                        b.push_str("| ");
                    } else {
                        b.push_str("  ");
                    }
                }
                if current_child != min_child {
                    b.push_str(
                        format!(
                            "├─{}",
                            self.g.token_raw.get(&current.children.get(current_child as usize).unwrap().symbol).unwrap()
                        )
                        .as_str(),
                    );
                } else {
                    b.push_str(
                        format!(
                            "└─{}",
                            self.g.token_raw.get(&current.children.get(current_child as usize).unwrap().symbol).unwrap()
                        )
                        .as_str(),
                    );
                }
                info!("{}", b);
                b.clear();
                if !current.children.get(current_child as usize).unwrap().children.is_empty() {
                    node_stack.push(current);
                    let child = current.children.get(current_child as usize).unwrap();
                    current_child -= 1;
                    node_stack.push(child);
                    child_count_stack.push((current_child, min_child));
                    child_count_stack.push(((child.children.len() - 1) as i32, 0));
                    break;
                }
                current_child -= 1;
            }
        }
    }
}

type Nd = (usize, String);
type Ed = (Nd, Nd);
struct Graph {
    nodes: Vec<String>,
    edges: Vec<(usize, usize)>,
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

pub fn render<W: Write>(ast: Box<AstNode>, output: &mut W) {
    let mut nodes: Vec<String> = Vec::new();
    let mut edges = Vec::new();

    nodes.push("Module".to_string());
    let mut stack: Vec<(Box<AstNode>, usize)> = vec![(ast, 0)];

    while let Some((current, id)) = stack.pop() {
        let mut push_node = |id, str: String, node: Box<AstNode>| {
            nodes.push(str);
            let child = nodes.len() - 1;
            edges.push((id, child));
            stack.push((node, child));
        };

        match *current {
            AstNode::Binary(left, _, right) => {
                push_node(id, format!("{:?}", &left), left);
                push_node(id, format!("{:?}", &right), right);
            }
            AstNode::Unary(_, expr) => {
                push_node(id, format!("{:?}", &expr), expr);
            }
            AstNode::Number(_) => {}
            AstNode::String(_) => {}
            AstNode::Name(_) => {}
            AstNode::ExprList(expr_list) => {
                for x in expr_list {
                    push_node(id, format!("{:?}", x), Box::from(x));
                }
            }
            AstNode::Assign(name, expr) => {
                push_node(id, format!("{:?}", name), name);
                push_node(id, format!("{:?}", expr), expr);
            }
            AstNode::Let(name, _, expr) => {
                push_node(id, format!("{:?}", name), name);
                if let Some(expr) = expr {
                    push_node(id, format!("{:?}", expr), expr);
                }
            }
            AstNode::Module(stmts) => {
                push_node(id, format!("{:?}", stmts), stmts);
            }
            AstNode::Function(_, param, stmts) => {
                if let Some(p) = param {
                    push_node(id, format!("Parameters: {:?}", p), p);
                }
                if let Some(stmts) = stmts {
                    push_node(id, format!("{:?}", stmts), stmts);
                }
            }
            AstNode::FunctionCall(expr, args) => {
                push_node(id, format!("{:?}", expr), expr);
                if let Some(args) = args {
                    push_node(id, format!("{:?}", args), args);
                }
            }
            AstNode::If(expr, stmts, else_or_elseif) => {
                push_node(id, format!("<B>Condition</B><BR/>{:?}", expr), expr);
                if let Some(stmts) = stmts {
                    push_node(id, format!("<B>If Body</B><BR/>{:?}", stmts), stmts);
                }
                if let Some(e) = else_or_elseif {
                    push_node(id, format!("{:?}", e), e);
                }
            }
            AstNode::For(var, expr, stmts) => {
                push_node(id, format!("Variable\n{:?}", var), var);
                push_node(id, format!("List\n{:?}", expr), expr);
                push_node(id, format!("{:?}", stmts), stmts);
            }
            AstNode::While(expr, stmts) => {
                push_node(id, format!("Condition\n{:?}", expr), expr);
                push_node(id, format!("{:?}", stmts), stmts);
            }
            AstNode::Return(expr) => {
                if let Some(expr) = expr {
                    push_node(id, format!("{:?}", expr), expr);
                }
            }
            AstNode::ElseIf(expr, stmts, else_or_elseif) => {
                push_node(id, format!("<B>Condition</B><BR/>{:?}", expr), expr);
                if let Some(stmts) = stmts {
                    push_node(id, format!("<B>Else If Body</B><BR/>{:?}", stmts), stmts);
                }
                if let Some(e) = else_or_elseif {
                    push_node(id, format!("{:?}", e), e);
                }
            }
            AstNode::Else(stmts) => {
                if let Some(stmts) = stmts {
                    push_node(id, format!("{:?}", stmts), stmts);
                }
            }
            AstNode::StatList(stmts) => {
                for x in stmts {
                    push_node(id, format!("{:?}", x), Box::from(x));
                }
            }
            AstNode::ExprThen(expr, stmt) => {
                push_node(id, format!("Condition\n{:?}", expr), expr);
                if let Some(e) = stmt {
                    push_node(id, format!("{:?}", e), e);
                }
            }
        }
    }

    let graph = Graph { nodes, edges };
    dot::render(&graph, output).unwrap()
}

pub fn render_block<W: Write>(ir: Block, w: &mut W) {
    w.write(
        r#"
digraph g {
    fontname="Helvetica,Arial,sans-serif"
    node [fontname="Helvetica,Arial,sans-serif"]
    edge [fontname="Helvetica,Arial,sans-serif"]
    graph [
        rankdir = "LR"
    ];
    node [
        fontsize = "16"
        shape = "ellipse"
        rankjustify=min
    ];
    edge [
    ];
    "root" [
    label = "root"
    shape = "record"                
    ];
"#
        .as_bytes(),
    )
    .unwrap();
    let mut stack = VecDeque::new();
    stack.push_front(&ir);
    let print_b = |b: &Block, w: &mut W| {
        let mut builder = String::new();
        let label = match &b.block_type {
            BlockType::Code(ref stmts) => {
                let mut stmt_to_string = |x: &Statement, first: bool| {
                    let mut prefix = "| ";
                    if first {
                        prefix = "";
                    }
                    match x {
                        crate::ir::Statement::Return(val) => {
                            if let Some(ref val) = val {
                                builder.push_str(format!("{}return {}\\l", prefix, val).as_str())
                            } else {
                                builder.push_str(format!("{}return\\l", prefix).as_str())
                            }
                        }
                        crate::ir::Statement::Let(l) => {
                            if let Some(ref val) = l.val {
                                builder.push_str(format!("{}{} = {} \\l", prefix, l.ident.name, val).as_str())
                            } else {
                                builder.push_str(format!("{}{}; \\l", prefix, l.ident.name).as_str())
                            }
                        }
                        _ => builder.push_str("?"),
                    }
                };
                let mut stmts = stmts.iter();
                if let Some(x) = stmts.next() {
                    stmt_to_string(&x, true);
                };
                while let Some(x) = stmts.next() {
                    stmt_to_string(&x, false);
                }
                &builder
            }
            BlockType::If(cond) => {
                builder = format!("{} ({})", &b.prefix, cond);
                &builder
            }
            _ => {
                println!("{}", &b.prefix);
                &b.prefix
            }
        };

        w.write(format!("\"{}\" [label = \"{}\"\n shape = \"record\"];\n", b.prefix, label).as_bytes())
            .unwrap();
    };

    while !stack.is_empty() {
        let current = stack.pop_front().unwrap();
        print_b(current, w);

        for b in &current.children {
            stack.push_front(b);
            w.write(format!("\"{}\" -> \"{}\" [];\n", current.prefix, b.prefix).as_bytes()).unwrap();
        }
    }

    w.write("\n}\n".as_bytes()).unwrap();
}
