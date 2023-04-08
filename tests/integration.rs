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
use log::debug;

#[test]
fn full_test() -> Result<(), Box<dyn Error>>{
    let mut now = Instant::now();

    let grammar = Grammar::from("json.g");

    debug!("Total Time to generate grammar : {:?}", now.elapsed());
    now = Instant::now();

    let mut tokens: Vec<u8> = Vec::new();
    {
        let file = File::open("data/full.json")?;
        let mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };
        thread::scope(|s| {
            let mut lexer = ParallelLexer::new(grammar.clone(), s, 1);
            let batch = lexer.new_batch();
            lexer.add_to_batch(&batch, &mmap[..], 0);
            tokens = lexer.collect_batch(batch);
            lexer.kill();
        });
    }

    let tree: Option<ParseTree>;
    {
        now = Instant::now();
        let mut parser = ParallelParser::new(grammar, 1);
        parser.parse(tokens.as_slice());
        parser.parse(&[parser.grammar.delim]);
        tree = Some(parser.collect_parse_tree().unwrap());
        debug!("Total Parsing Time: {:?}", now.elapsed());
    }

    let _ = tree;

    now = Instant::now();

    debug!("Total Time For ParseTree -> AST Conversion: {:?}", now.elapsed());
    Ok(())
}

#[test]
fn parallel_lexing() -> Result<(), Box<dyn Error>> {
    let grammar = Grammar::from("json.g");
    let threads = 12;
    let now = Instant::now();
    let file = File::open("data/full.json")?;
    let x: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };

    let mut indices = vec![];
    let step = 10;
    let mut i = 0;
    let mut prev = 0;

    while i < x.len() {
        if x[i] as char != ' ' && x[i] as char != '\n' {
            i += 1;
        } else {
            if i + 1 <= x.len() {
                i += 1;
            }
            indices.push((prev, i));
            prev = i;
            i += step;
        }
    }
    if prev < x.len() {
        indices.push((prev, x.len()));
    }

    let mut units = vec![];
    for i in indices {
        units.push(&x[i.0..i.1]);
    }

    debug!("Reading file : {:?}", now.elapsed());
    let mut now = Instant::now();

    thread::scope(|s| {
        let mut lexer = ParallelLexer::new(grammar, s, threads);
        let batch = lexer.new_batch();
        for task in units.iter().enumerate() {
            lexer.add_to_batch(&batch, task.1, task.0);
        }
        let output = lexer.collect_batch(batch);
        lexer.kill();
    });
    Ok(())
}
