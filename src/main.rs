#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
extern crate core;

use std::error::Error;
use std::fs::File;
use std::thread;
use std::time::Instant;

use memmap::MmapOptions;
use crate::grammar::Grammar;
use crate::lexer::json::JsonToken;
use crate::lexer::ParallelLexer;
use crate::parser::{ParallelParser, ParseTree};

mod error;
mod grammar;
mod parser;
mod integration;
mod lexer;

fn main() -> Result<(), Box<dyn Error>> {

    let mut now = Instant::now();
    let mut tokens: Vec<JsonToken> = Vec::new();
    {
        let file = File::open("json/500KB.json")?;
        let mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };
        thread::scope(|s| {
            let mut lexer = ParallelLexer::new(s, 1);
            let batch = lexer.new_batch();
            lexer.add_to_batch(&batch, &mmap[..], 0);
            tokens = lexer.collect_batch(batch, &mut now);
            lexer.kill();
        });
    }

    let tree: Option<ParseTree> = None;
    {
        now = Instant::now();
        let mut parser = ParallelParser::new(Grammar::<JsonToken>::json_grammar(), 1);
        parser.parse(tokens.as_slice());
        parser.parse(&[parser.grammar.delim]);
        let tree = Some(parser.collect_parse_tree().unwrap());
        println!("Total Parsing Time: {:?}", now.elapsed());
    }

    now = Instant::now();

    println!("Total Time For ParseTree -> AST Conversion: {:?}", now.elapsed());
    Ok(())
}
