#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

use crate::fern_ast::{AstNode, FernParseTree};
use crate::grammar::{OpGrammar, Token};
use crate::lexer::lua::{LuaLexer, LuaLexerState};
use crate::lexer::LexerInterface;
use crate::lexer::ParallelLexer;
use crate::print::{render, render_block};
pub use grammar::reader;
use std::collections::LinkedList;
use std::error::Error;
use std::fs::File;
use std::thread;
use std::time::{Duration, Instant};
extern crate console_error_panic_hook;

pub mod analysis;
pub mod cfg;
pub mod grammar;
pub mod ir;
pub mod lexer;
pub mod parser;
pub mod server;
pub mod slab;

use crate::lexer::fern::{FernData, FernLexer, FernLexerState};
use log::{debug, info};
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
pub fn compile_fern(input: &str) -> String {
    use lexer::fern::FernTokens;

    wasm_logger::init(wasm_logger::Config::default());
    console_error_panic_hook::set_once();

    // let mut now = Instant::now();
    let mut raw = RawGrammar::new(COMP_TIME_GRAMMAR).unwrap();
    raw.delete_repeated_rhs().unwrap();
    let grammar = OpGrammar::new(raw).unwrap();
    // grammar.to_file("data/grammar/fern.g");

    let mut lexer = crate::FernLexer::new(grammar.clone(), FernLexerState::Start);
    for x in input.chars() {
        lexer.consume(&(x as u8));
    }

    let (_, temp) = lexer.take();
    let mut tokens: LinkedList<Vec<(u16, FernData)>> = LinkedList::new();
    tokens.push_front(temp);

    let (tree, time): (FernParseTree, Duration) = {
        let mut parser = ParallelParser::new(grammar.clone(), 1);
        parser.parse(tokens);
        parser.parse(LinkedList::from([vec![(grammar.delim, FernData::NoData)]]));
        let time = parser.time_spent_rule_searching.clone();
        (parser.collect_parse_tree().unwrap(), time)
    };

    tree.print();
    // info!("Total Time to parse: {:?}", now.elapsed());
    // info!("└─Total Time spent rule-searching: {:?}", time);
    // now = Instant::now();

    let ast: Box<AstNode> = Box::from(tree.build_ast().unwrap());
    let mut buf = Vec::new();
    render(ast.clone(), &mut buf);
    let string = String::from_utf8(buf).unwrap();

    return string;
}
