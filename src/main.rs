#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
extern crate core;

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
use core::parser::{ParallelParser, ParseTree};
use memmap::MmapOptions;
use std::thread::park;
use flexi_logger::Logger;

fn json() -> Result<(), Box<dyn Error>> {
    let mut now = Instant::now();
    let grammar = OpGrammar::from("data/grammar/json.g");
    info!("Total Time to get grammar : {:?}", now.elapsed());
    now = Instant::now();
    let file = File::open("data/json/100MB.json").unwrap();
    let mut memmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
    let chunks = split_mmap_into_chunks(&mut memmap, 6000).unwrap();
    info!("Total time to load and split up file: {:?}", now.elapsed());
    now = Instant::now();

    let _ = thread::scope(|s| {
        let mut lexer: ParallelLexer<JsonLexerState, JsonLexer> = ParallelLexer::new(
            &grammar,
            s,
            16,
            &[JsonLexerState::Start, JsonLexerState::InString],
            JsonLexerState::Start,
        );
        let batch = lexer.new_batch();
        for task in chunks.iter().enumerate() {
            lexer.add_to_batch(&batch, task.1, task.0);
        }
        let tokens = lexer.collect_batch(batch);
        lexer.kill();
        tokens
    });
    info!("Total Time to lex: {:?}", now.elapsed());
    // now = Instant::now();
    // let mut now = Instant::now();
    // let grammar = OpGrammar::from("data/grammar/json.g");
    // info!("Total Time to get grammar : {:?}", now.elapsed());
    // now = Instant::now();
    //
    // let tokens: LinkedList<Vec<Token>> = {
    //     let file = File::open("data/test.json")?;
    //     let mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };
    //     thread::scope(|s| {
    //         let mut lexer: ParallelLexer<JsonLexerState, JsonLexer> =
    //             ParallelLexer::new(&grammar, s, 1, &[JsonLexerState::Start, JsonLexerState::InString], JsonLexerState::Start);
    //         let batch = lexer.new_batch();
    //         lexer.add_to_batch(&batch, &mmap[..], 0);
    //         let tokens = lexer.collect_batch(batch);
    //         lexer.kill();
    //         tokens
    //     })
    // };
    //
    // info!("Total Time to lex: {:?}", now.elapsed());
    // now = Instant::now();
    //
    // let (tree, time): (ParseTree, Duration) = {
    //     let mut parser = ParallelParser::new(grammar.clone(), 1);
    //     parser.parse(tokens);
    //     parser.parse(LinkedList::from([vec![grammar.delim]]));
    //     let time = parser.time_spent_rule_searching.clone();
    //     (parser.collect_parse_tree().unwrap(), time)
    // };
    //
    // tree.print();
    // info!("Total Time to parse: {:?}", now.elapsed());
    // info!("└─Total Time spent rule-searching: {:?}", time);
    //
    // now = Instant::now();
    // info!(
    //     "Total Time to transform ParseTree -> AST Conversion: {:?}",
    //     now.elapsed()
    // );
    Ok(())
}

fn rust() -> Result<(), Box<dyn Error>> {
    let mut now = Instant::now();
    let mut raw = RawGrammar::from("data/grammar/fern.g")?;
    raw.delete_repeated_rhs()?;
    let grammar = OpGrammar::new(raw)?;
    grammar.to_file("data/grammar/fern-fnf.g");

    info!("Total Time to get grammar : {:?}", now.elapsed());
    now = Instant::now();

    let tokens: LinkedList<Vec<Token>> = {
        let file = File::open("data/test.fern")?;
        let mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };
        thread::scope(|s| {
            let mut lexer: ParallelLexer<FernLexerState, FernLexer> =
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

    let (tree, time): (ParseTree, Duration) = {
        let mut parser = ParallelParser::new(grammar.clone(), 1);
        parser.parse(tokens);
        parser.parse(LinkedList::from([vec![grammar.delim]]));
        let time = parser.time_spent_rule_searching.clone();
        (parser.collect_parse_tree().unwrap(), time)
    };

    tree.print();
    info!("Total Time to parse: {:?}", now.elapsed());
    info!("└─Total Time spent rule-searching: {:?}", time);

    now = Instant::now();
    info!(
        "Total Time to transform ParseTree -> AST Conversion: {:?}",
        now.elapsed()
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    Logger::try_with_str("trace, core::grammar = info")?.start_with_specfile("log.toml")?;
    json()?;
    Ok(())
}
