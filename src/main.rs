#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
extern crate core;

use std::collections::LinkedList;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::Instant;
use log::{info, LevelFilter};

use core::grammar::Grammar;
use core::grammar::Token;
use core::lexer::*;
use core::lexer::{lua::*, json::*, fern::*};
use core::parser::{ParallelParser, ParseTree};
use memmap::MmapOptions;

fn lua() -> Result<(), Box<dyn Error>> {
    let mut now = Instant::now();
    let grammar = match File::open(".cached-grammar") {
        Ok(f) => {
            info!("Using cached grammar from file : .cached-grammar");
            let grammar = ciborium::de::from_reader::<'_, Grammar, _>(f).unwrap();
            grammar
        }
        Err(_) => {
            info!("Generating grammar from scratch...");
            let grammar = Grammar::from("data/grammar/lua.g");
            let f = File::create(".cached-grammar").unwrap();
            info!("Grammar saved to .cached-grammar");
            ciborium::ser::into_writer(&grammar, f).unwrap();
            grammar
        }
    };

    info!("Total Time to get grammar : {:?}", now.elapsed());
    now = Instant::now();

    let tokens: LinkedList<Vec<Token>> = {
        let file = File::open("data/test.lua")?;
        let mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };
        thread::scope(|s| {
            let mut lexer: ParallelLexer<LuaLexerState, LuaLexer> = ParallelLexer::new (
                grammar.clone(),
                s,
                1,
                &[LuaLexerState::Start],
                LuaLexerState::Start,
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

fn main() -> Result<(), Box<dyn Error>> {
    let config: simplelog::Config = simplelog::ConfigBuilder::new()
        .set_time_level(LevelFilter::Off)
        .set_target_level(LevelFilter::Off)
        .set_thread_level(LevelFilter::Off)
        .build();
    let _ = simplelog::SimpleLogger::init(LevelFilter::Trace, config);
    // core::server::start_http_server();
    lua()?;
    Ok(())
}
