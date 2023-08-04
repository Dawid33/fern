#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
#![allow(ambiguous_glob_reexports)]
extern crate core;

use core::print::{render, render_block};
pub use core::*;
use std::borrow::Cow;

use core::lexer::fern::FernData;
use crossbeam_queue::SegQueue;
use log::{info, LevelFilter};
use std::collections::LinkedList;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::{Duration, Instant};

use crate::cfg::ControlFlowGraph;
use crate::parser::fern_ast::AstNode;
use crate::parser::json::JsonParseTree;
use core::grammar::OpGrammar;
use core::grammar::RawGrammar;
use core::grammar::Token;
use core::lexer::*;
use core::lexer::{fern::*, json::*, lua::*};
use core::parser::fern_ast::FernParseTree;
use core::parser::ParallelParser;
use flexi_logger::Logger;
use memmap::MmapOptions;
use std::ops::Deref;
use std::thread::{current, park};
// use tungstenite::protocol::frame::coding::Data;

fn json() -> Result<(), Box<dyn Error>> {
    let mut now = Instant::now();
    let mut raw = RawGrammar::from("data/grammar/json.g")?;
    raw.delete_repeated_rhs()?;
    let grammar = OpGrammar::new(raw)?;
    grammar.to_file("data/grammar/json-fnf.g");
    info!("Total Time to get grammar : {:?}", now.elapsed());
    now = Instant::now();

    let tokens: LinkedList<Vec<(Token, JsonData)>> = {
        let file = File::open("data/json/10KB.json")?;
        let mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };
        thread::scope(|s| {
            let mut lexer: ParallelLexer<JsonLexerState, JsonLexer, JsonData> =
                ParallelLexer::new(&grammar, s, 16, &[JsonLexerState::Start, JsonLexerState::InString], JsonLexerState::Start);
            let batch = lexer.new_batch();
            lexer.add_to_batch(&batch, &mmap[..], 0);
            let tokens = lexer.collect_batch(batch);
            lexer.kill();
            tokens
        })
    };

    info!("Total Time to lex: {:?}", now.elapsed());
    now = Instant::now();

    let (tree, time): (JsonParseTree, Duration) = {
        let mut parser = ParallelParser::new(grammar.clone(), 1);
        parser.parse(tokens);
        parser.parse(LinkedList::from([vec![(grammar.delim, JsonData::NoData)]]));
        let time = parser.time_spent_rule_searching.clone();
        (parser.collect_parse_tree().unwrap(), time)
    };

    tree.print();
    info!("Total Time to parse: {:?}", now.elapsed());
    info!("└─Total Time spent rule-searching: {:?}", time);

    now = Instant::now();
    info!("Total Time to transform ParseTree -> AST Conversion: {:?}", now.elapsed());
    Ok(())
}

fn fern() -> Result<(), Box<dyn Error>> {
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

    let ast: Box<AstNode> = Box::from(tree.build_ast().unwrap());
    use std::fs::File;
    let mut f = File::create("ast.dot").unwrap();
    render(ast.clone(), &mut f);

    info!("Total Time to transform ParseTree -> AST: {:?}", now.elapsed());
    now = Instant::now();

    let graph = ir::Block::from(ast).unwrap();
    let mut f_ir = File::create("ir.dot").unwrap();
    render_block(graph, &mut f_ir);
    info!("Total Time to transform AST -> IR: {:?}", now.elapsed());
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    Logger::try_with_str("trace, core::grammar = info")?.start_with_specfile("log.toml")?;
    fern()?;
    Ok(())
}
