extern crate core;

use std::collections::LinkedList;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::Instant;

use core::grammar::Grammar;
use core::lexer::ParallelLexer;
use core::parser::{ParallelParser, ParseTree};
use log::{debug, info};
use memmap::MmapOptions;

#[test]
fn full_test() -> Result<(), Box<dyn Error>> {
    let config: simplelog::Config = simplelog::ConfigBuilder::new()
        .set_time_level(simplelog::LevelFilter::Off)
        .set_target_level(simplelog::LevelFilter::Off)
        .set_thread_level(simplelog::LevelFilter::Off)
        .build();
    let _ = simplelog::SimpleLogger::init(simplelog::LevelFilter::Info, config);
    let mut now = Instant::now();
    let grammar = Grammar::from("json.g");
    info!("Total Time to generate grammar : {:?}", now.elapsed());
    now = Instant::now();

    let mut tokens: LinkedList<Vec<u8>> = LinkedList::new();
    {
        let file = File::open("json/100MB.json")?;
        let mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };
        info!("Total time to load file: {:?}", now.elapsed());
        now = Instant::now();
        thread::scope(|s| {
            let mut lexer = ParallelLexer::new(grammar.clone(), s, 1);
            let batch = lexer.new_batch();
            lexer.add_to_batch(&batch, &mmap[..], 0);
            tokens = lexer.collect_batch(batch);
            lexer.kill();
        });
    }
    info!("Total Lexing Time: {:?}", now.elapsed());

    // let tree: ParseTree = {
    //     now = Instant::now();
    //     let mut parser = ParallelParser::new(grammar.clone(), 1);
    //     parser.parse(tokens);
    //     parser.parse(LinkedList::from([vec![grammar.delim]]));
    //     parser.collect_parse_tree().unwrap()
    // };
    //
    // debug!("Total Parsing Time: {:?}", now.elapsed());
    //
    // let _ = tree;
    //
    // now = Instant::now();
    //
    // debug!("Total Time For ParseTree -> AST Conversion: {:?}", now.elapsed());
    Ok(())
}

#[test]
fn parallel_lexing() -> Result<(), Box<dyn Error>> {
    let config: simplelog::Config = simplelog::ConfigBuilder::new()
        .set_time_level(simplelog::LevelFilter::Off)
        .set_target_level(simplelog::LevelFilter::Off)
        .set_thread_level(simplelog::LevelFilter::Off)
        .build();
    let _ = simplelog::SimpleLogger::init(simplelog::LevelFilter::Info, config);

    let now = Instant::now();
    let grammar = Grammar::from("json.g");
    info!("Total Time to generate grammar : {:?}", now.elapsed());
    let now = Instant::now();

    let file = File::open("json/100KB.json").unwrap();
    let mut memmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
    info!("Total time to load file: {:?}", now.elapsed());
    let mut now = Instant::now();

    let chunks = core::lexer::split_mmap_into_chunks(&mut memmap, 6000).unwrap();

    let tokens = thread::scope(|s| {
        let mut lexer = ParallelLexer::new(grammar.clone(), s, 16);
        let batch = lexer.new_batch();
        for task in chunks.iter().enumerate() {
            lexer.add_to_batch(&batch, task.1, task.0);
        }
        let output = lexer.collect_batch(batch);
        lexer.kill();
        output
    });

    info!("Total Lexing Time: {:?}", now.elapsed());

    // let tree: ParseTree = {
    //     now = Instant::now();
    //     let mut parser = ParallelParser::new(grammar.clone(), 1);
    //     parser.parse(tokens);
    //     parser.parse(LinkedList::from([vec![grammar.delim]]));
    //     parser.collect_parse_tree().unwrap()
    // };
    //
    // info!("Total Parsing Time: {:?}", now.elapsed());
    Ok(())
}
