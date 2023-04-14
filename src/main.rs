#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
extern crate core;

use std::collections::LinkedList;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::Instant;

use core::grammar::Grammar;
use core::parser::{ParallelParser, ParseTree};
use core::lexer::{json::*};
use core::lexer::*;
use log::{debug, info};
use memmap::MmapOptions;

fn main() -> Result<(), Box<dyn Error>> {
    let mut now = Instant::now();
    let config: simplelog::Config = simplelog::ConfigBuilder::new()
        .set_time_level(simplelog::LevelFilter::Off)
        .set_target_level(simplelog::LevelFilter::Off)
        .set_thread_level(simplelog::LevelFilter::Off)
        .build();
    let _ = simplelog::SimpleLogger::init(simplelog::LevelFilter::Debug, config);

    let grammar = Grammar::from("data/grammar/json.g");
    info!("Total Time to generate grammar : {:?}", now.elapsed());
    now = Instant::now();

    let tokens: LinkedList<Vec<u8>> = {
        let file = File::open("data/full.json")?;
        let mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };
        thread::scope(|s| {
            let mut lexer: ParallelLexer<JsonLexerState, JsonLexer> = ParallelLexer::new(grammar.clone(), s, 1, &[JsonLexerState::Start, JsonLexerState::InString], JsonLexerState::Start);
            let batch = lexer.new_batch();
            lexer.add_to_batch(&batch, &mmap[..], 0);
            let tokens = lexer.collect_batch(batch);
            lexer.kill();
            tokens
        })
    };

    info!("Total Time to lex: {:?}", now.elapsed());
    now = Instant::now();
    info!("{:?}", tokens);

    let tree: ParseTree = {
        let mut parser = ParallelParser::new(grammar.clone(), 1);
        parser.parse(tokens);
        parser.parse(LinkedList::from([vec![grammar.delim]]));
        parser.collect_parse_tree().unwrap()
    };

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
