#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
extern crate core;

use std::collections::LinkedList;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::Instant;
use tracing::{event_enabled, info, Level, span};
use tracing_subscriber::FmtSubscriber;

use core::grammar::Grammar;
use core::grammar::Token;
use core::lexer::*;
use core::lexer::{lua::*, json::*};
use core::parser::{ParallelParser, ParseTree};
use memmap::MmapOptions;

fn main() -> Result<(), Box<dyn Error>> {
    let mut now = Instant::now();
    let subscriber = FmtSubscriber::builder()
        .without_time()
        .with_target(false)
        .with_max_level(Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let span = span!(Level::TRACE, "grammar");
    let span = span.enter();
    let grammar = Grammar::from("data/grammar/lua.g");
    drop(span);
    info!("Total Time to generate grammar : {:?}", now.elapsed());
    now = Instant::now();

    let tokens: LinkedList<Vec<Token>> = {
        let file = File::open("data/test.lua")?;
        let mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };
        thread::scope(|s| {
            let span = span!(Level::TRACE, "lexer");
            span.enter();
            let mut lexer: ParallelLexer<FernLexerState, FernLexer> = ParallelLexer::new(
                grammar.clone(),
                s,
                1,
                &[FernLexerState::Start],
                FernLexerState::Start,
            );
            let batch = lexer.new_batch();
            lexer.add_to_batch(&batch, &mmap[..], 0);
            let tokens = lexer.collect_batch(batch);
            lexer.kill();
            tokens
        })
    };


    info!("Total Time to lex: {:?}", now.elapsed());
    now = Instant::now();

    let span = span!(Level::TRACE, "parser");
    let span = span.enter();
    let tree: ParseTree = {
        let mut parser = ParallelParser::new(grammar.clone(), 1);
        parser.parse(tokens);
        parser.parse(LinkedList::from([vec![grammar.delim]]));
        parser.collect_parse_tree().unwrap()
    };
    drop(span);

    tree.print();
    let _json: core::parser::json::JsonValue = tree.into();
    info!("{:?}", _json);
    info!("Total Time to parse: {:?}", now.elapsed());
    now = Instant::now();

    info!(
        "Total Time to transform ParseTree -> AST Conversion: {:?}",
        now.elapsed()
    );
    Ok(())
}
