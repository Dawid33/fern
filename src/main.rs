#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
extern crate core;

use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::Instant;

use memmap::MmapOptions;
use core::grammar::Grammar;
use core::lexer::ParallelLexer;
use core::parser::{ParallelParser, ParseTree};
use log::{debug, info};

fn main() -> Result<(), Box<dyn Error>> {
    let mut now = Instant::now();
    let _ = simplelog::SimpleLogger::init(simplelog::LevelFilter::Info, simplelog::Config::default());

    let grammar = Grammar::from("json.g");
    info!("Total Time to generate grammar : {:?}", now.elapsed());
    now = Instant::now();

    let mut tokens: Vec<u8> = Vec::new();
    {
        let file = File::open("json/1MB.json")?;
        let mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };
        thread::scope(|s| {
            let mut lexer = ParallelLexer::new(grammar.clone(), s, 1);
            let batch = lexer.new_batch();
            lexer.add_to_batch(&batch, &mmap[..], 0);
            tokens = lexer.collect_batch(batch);
            lexer.kill();
        });
    }

    // let tree: Option<ParseTree>;
    // {
    //     now = Instant::now();
    //     let mut parser = ParallelParser::new(grammar, 1);
    //     parser.parse(tokens.as_slice());
    //     parser.parse(&[parser.grammar.delim]);
    //     tree = Some(parser.collect_parse_tree().unwrap());
    //     println!("Total Parsing Time: {:?}", now.elapsed());
    // }
    //
    // let _ = tree;

    now = Instant::now();

    info!("Total Time For ParseTree -> AST Conversion: {:?}", now.elapsed());
    Ok(())
}
