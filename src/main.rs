#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
extern crate core;

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

fn lua() -> Result<(), Box<dyn Error>> {
    let mut now = Instant::now();
    let grammar = match File::open("data/grammar/lua-fnf.g") {
        Ok(f) => {
            drop(f);
            let grammar = OpGrammar::from("data/grammar/lua-fnf.g");
            grammar
        }
        Err(_) => {
            let mut raw = RawGrammar::from("data/grammar/lua.g")?;
            raw.delete_repeated_rhs()?;
            let grammar = OpGrammar::new(raw)?;
            grammar.to_file("data/grammar/lua-fnf.g");
            grammar
        }
    };

    info!("Total Time to get grammar : {:?}", now.elapsed());
    now = Instant::now();

    let tokens: LinkedList<Vec<Token>> = {
        let file = File::open("data/test.lua")?;
        let mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };
        thread::scope(|s| {
            let mut lexer: ParallelLexer<LuaLexerState, LuaLexer> =
                ParallelLexer::new(grammar.clone(), s, 1, &[LuaLexerState::Start], LuaLexerState::Start);
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
    lua()?;
    Ok(())
}
