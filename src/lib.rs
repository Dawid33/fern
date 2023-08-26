#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

use crate::fern_ast::{AstNode, FernParseTree};
use crate::grammar::{OpGrammar, Token};
use crate::lexer::lua::{LuaLexer, LuaLexerState};
use crate::lexer::ParallelLexer;
use crate::print::{render, render_block};
use std::collections::LinkedList;
use std::error::Error;
use std::fs::File;
use std::thread;
use std::time::{Duration, Instant};

pub mod analysis;
pub mod cfg;
pub mod grammar;
pub mod ir;
pub mod lexer;
pub mod parser;
pub mod server;
pub mod slab;

use crate::lexer::fern::{FernData, FernLexer, FernLexerState};
pub use grammar::*;
use log::info;
pub use parser::*;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[cfg(target_arch = "wasm32")]
extern "C" {
    fn alert(s: &str);
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn greet() {
    alert("Hello, wasm-game-of-life!");
}

#[cfg(target_arch = "wasm32")]
const COMP_TIME_GRAMMAR: &'static str = include_str!("../data/grammar/fern.g");

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

#[wasm_bindgen]
#[cfg(target_arch = "wasm32")]
pub fn compile_fern(input: &str) {
    wasm_logger::init(wasm_logger::Config::default());
    // let mut now = Instant::now();
    let mut raw = RawGrammar::new(COMP_TIME_GRAMMAR).unwrap();
    raw.delete_repeated_rhs().unwrap();
    let grammar = OpGrammar::new(raw).unwrap();
    // grammar.to_file("data/grammar/fern.g");

    // info!("Total Time to get grammar : {:?}", now.elapsed());
    // now = Instant::now();

    // let tokens: LinkedList<Vec<(Token, FernData)>> = {
    //     thread::scope(|s| {
    //         let mut lexer: ParallelLexer<FernLexerState, FernLexer, FernData> =
    //             ParallelLexer::new(&grammar, s, 1, &[FernLexerState::Start], FernLexerState::Start);
    //         let batch = lexer.new_batch();
    //         lexer.add_to_batch(&batch, input.as_bytes(), 0);
    //         let tokens = lexer.collect_batch(batch);
    //         lexer.kill();
    //         tokens
    //     })
    // };

    // info!("Total Time to lex: {:?}", now.elapsed());
    // now = Instant::now();

    // let (tree, time): (FernParseTree, Duration) = {
    //     let mut parser = ParallelParser::new(grammar.clone(), 1);
    //     parser.parse(tokens);
    //     parser.parse(LinkedList::from([vec![(grammar.delim, FernData::NoData)]]));
    //     let time = parser.time_spent_rule_searching.clone();
    //     (parser.collect_parse_tree().unwrap(), time)
    // };

    // tree.print();
    // info!("Total Time to parse: {:?}", now.elapsed());
    // info!("└─Total Time spent rule-searching: {:?}", time);
    // now = Instant::now();

    // let ast: Box<AstNode> = Box::from(tree.build_ast().unwrap());
    // use std::fs::File;
    // let mut f = File::create("ast.dot").unwrap();
    // render(ast.clone(), &mut f);

    // info!("Total Time to transform ParseTree -> AST: {:?}", now.elapsed());
    // now = Instant::now();

    // let graph = ir::Block::from(ast).unwrap();
    // let mut f_ir = File::create("ir.dot").unwrap();
    // render_block(graph, &mut f_ir);
    // info!("Total Time to transform AST -> IR: {:?}", now.elapsed());
}
