#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
extern crate core;

use std::borrow::Cow;
pub use core::*;

use core::lexer::fern::FernData;
use crossbeam_queue::SegQueue;
use log::{info, LevelFilter};
use std::collections::LinkedList;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::{Duration, Instant};

use core::grammar::OpGrammar;
use core::grammar::RawGrammar;
use core::grammar::Token;
use core::lexer::*;
use core::lexer::{fern::*, json::*, lua::*};
use core::parser::{ParallelParser, FernParseTree};
use std::ops::Deref;
use memmap::MmapOptions;
use std::thread::{current, park};
use flexi_logger::Logger;
use tungstenite::protocol::frame::coding::Data;
use crate::parser::fern::AstNode;

type Nd = (usize, String);
type Ed = (Nd, Nd);
struct Graph { nodes: Vec<String>, edges: Vec<(usize,usize)> }


// fn json() -> Result<(), Box<dyn Error>> {
//     let mut now = Instant::now();
//     let grammar = OpGrammar::from("data/grammar/json.g");
//     info!("Total Time to get grammar : {:?}", now.elapsed());
//     now = Instant::now();
//
//     let tokens: LinkedList<Vec<(Token, JsonData)>> = {
//         let file = File::open("data/test.json")?;
//         let mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };
//         thread::scope(|s| {
//             let mut lexer: ParallelLexer<JsonLexerState, JsonLexer, JsonData> =
//                 ParallelLexer::new(&grammar, s, 1, &[JsonLexerState::Start, JsonLexerState::InString], JsonLexerState::Start);
//             let batch = lexer.new_batch();
//             lexer.add_to_batch(&batch, &mmap[..], 0);
//             let tokens = lexer.collect_batch(batch);
//             lexer.kill();
//             tokens
//         })
//     };
//
//     info!("Total Time to lex: {:?}", now.elapsed());
//     now = Instant::now();
//
//     let (tree, time): (ParseTree, Duration) = {
//         let mut parser = ParallelParser::new(grammar.clone(), 1);
//         parser.parse(tokens);
//         parser.parse(LinkedList::from([vec![grammar.delim]]));
//         let time = parser.time_spent_rule_searching.clone();
//         (parser.collect_parse_tree().unwrap(), time)
//     };
//
//     tree.print();
//     info!("Total Time to parse: {:?}", now.elapsed());
//     info!("└─Total Time spent rule-searching: {:?}", time);
//
//     now = Instant::now();
//     info!(
//         "Total Time to transform ParseTree -> AST Conversion: {:?}",
//         now.elapsed()
//     );
//     Ok(())
// }

fn rust() -> Result<(), Box<dyn Error>> {
    let mut now = Instant::now();
    let mut raw = RawGrammar::from("data/grammar/fern.g")?;
    raw.delete_repeated_rhs()?;
    let grammar = OpGrammar::new(raw)?;
    grammar.to_file("data/grammar/fern-fnf.g");

    info!("Total Time to get grammar : {:?}", now.elapsed());
    now = Instant::now();

    let tokens: LinkedList<Vec<(Token, FernData)>> = {
        let file = File::open("data/test.fern")?;
        let mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };
        thread::scope(|s| {
            let mut lexer: ParallelLexer<FernLexerState, FernLexer, FernData> =
                ParallelLexer::new(&grammar, s, 1, &[FernLexerState::Start], FernLexerState::Start);
            let batch = lexer.new_batch();
            lexer.add_to_batch(&batch, &mmap[..], 0);
            let tokens = lexer.collect_batch(batch);
            lexer.kill();
            tokens
        })
    };

    info!("Total Time to lex: {:?}", now.elapsed());
    now = Instant::now();

    let (tree, time): (FernParseTree, Duration) = {
        let mut parser = ParallelParser::new(grammar.clone(), 1);
        parser.parse(tokens);
        parser.parse(LinkedList::from([vec![(grammar.delim, FernData::NoData)]]));
        let time = parser.time_spent_rule_searching.clone();
        (parser.collect_parse_tree().unwrap(), time)
    };

    tree.print();
    info!("Total Time to parse: {:?}", now.elapsed());
    info!("└─Total Time spent rule-searching: {:?}", time);

    now = Instant::now();

    let ast: AstNode = tree.build_ast().unwrap();
    use std::fs::File;
    let mut f = File::create("ast.dot").unwrap();
    render(ast, &mut f);

    info!(
        "Total Time to transform ParseTree -> AST Conversion: {:?}",
        now.elapsed()
    );
    Ok(())
}


impl<'a> dot::Labeller<'a, Nd, Ed> for Graph {
    fn graph_id(&'a self) -> dot::Id<'a> { dot::Id::new("example3").unwrap() }
    fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
        dot::Id::new(format!("N{}", n.0)).unwrap()
    }
    fn node_label(&self, n: &Nd) -> dot::LabelText {
        let &(i, _) = n;
        dot::LabelText::LabelStr(self.nodes[i].clone().into())
    }
    fn edge_label(&self, _: &Ed) -> dot::LabelText {
        dot::LabelText::LabelStr("".into())
    }
}

impl<'a> dot::GraphWalk<'a, Nd, Ed> for Graph {
    fn nodes(&'a self) -> dot::Nodes<'a,Nd> {
        let mut new_nodes = self.nodes.clone().into_iter().enumerate().collect();
        Cow::Owned(new_nodes)
    }
    fn edges(&'a self) -> dot::Edges<'a,Ed> {
        self.edges.iter()
            .map(|&(i,j)|((i, self.nodes[i].clone()),
                          (j, self.nodes[j].clone())))
            .collect()
    }
    fn source(&self, e: &Ed) -> Nd { e.0.clone() }
    fn target(&self, e: &Ed) -> Nd { e.1.clone() }
}

pub fn render<W: Write>(ast: AstNode, output: &mut W) {
    let mut nodes: Vec<String> = Vec::new();
    let mut edges = Vec::new();

    nodes.push("Module".to_string());
    let mut stack: Vec<(Box<AstNode>, usize)> = vec!((Box::from(ast), 0));


    while let Some((current, id)) = stack.pop() {
        let mut push_node = |id, node: Box<AstNode>| {
            nodes.push(format!("{:?}", node));
            let child = nodes.len() - 1;
            edges.push((id, child));
            stack.push((node, child));
        };

        match *current {
            AstNode::Binary(left, op, right) => {
                push_node(id, left);
                push_node(id, right);
            }
            AstNode::Unary(op, expr) => {
                push_node(id, expr);
            }
            AstNode::Number(_) => {}
            AstNode::String(_) => {}
            AstNode::Name(_) => {}
            AstNode::NameList(name_list) => {
                for x in name_list {
                    push_node(id, Box::from(x));
                }
            }
            AstNode::Assign(name, expr) => {
                push_node(id, name);
                push_node(id, expr);
            }
            AstNode::Let(name, _, expr) => {
                push_node(id, name);
                push_node(id, expr);
            }
            AstNode::Module(stmts) => {
                for x in stmts {
                    push_node(id, Box::from(x));
                }
            }
            AstNode::Function(_, param, stmts) => {
                push_node(id, param);
                for x in stmts {
                    push_node(id, Box::from(x));
                }
            }
            AstNode::If(expr, stmts, else_or_elseif) => {
                push_node(id, expr);
                for x in stmts {
                    push_node(id, Box::from(x));
                }
                if let Some(e) = else_or_elseif {
                    push_node(id, e);
                }
            }
            AstNode::For(var, expr, stmts) => {
                push_node(id, var);
                push_node(id, expr);
                for x in stmts {
                    push_node(id, Box::from(x));
                }
            }
            AstNode::While(expr, stmts) => {
                push_node(id, expr);
                for x in stmts {
                    push_node(id, Box::from(x));
                }
            },
            AstNode::Return(expr) => {
                if let Some(expr) = expr {
                    push_node(id, expr);
                }
            }
            AstNode::ElseIf(expr, stmts, else_or_elseif) => {
                push_node(id, expr);
                for x in stmts {
                    push_node(id, Box::from(x));
                }
                if let Some(e) = else_or_elseif {
                    push_node(id, e);
                }
            }
            AstNode::Else(stmts) => {
                for x in stmts {
                    push_node(id, Box::from(x));
                }
            }
        }
    }


    let graph = Graph { nodes: nodes, edges: edges };
    dot::render(&graph, output).unwrap()
}

fn main() -> Result<(), Box<dyn Error>> {
    Logger::try_with_str("trace, core::grammar = info")?.start_with_specfile("log.toml")?;
    rust()?;
    Ok(())
}
