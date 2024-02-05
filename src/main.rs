#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
#![allow(ambiguous_glob_reexports)]
extern crate core;

use crate::fern::FernLexer;
use crate::grammar::opg::{OpGrammar, RawGrammar};
use crate::lexer::{Data, ParallelLexer, Token};
use crate::parser::{ParallelParser, ParseTree};
use crossbeam_queue::SegQueue;
use fern::FernParseTree;
use flexi_logger::Logger;
use lexer::LexerInterface;
use log::{debug, info, trace, warn, LevelFilter};
use memmap::Mmap;
use memmap::MmapOptions;
use regex_syntax::{hir::Hir, parse};
use std::borrow::Cow;
use std::collections::{HashMap, LinkedList};
use std::error::Error;
use std::fs::{self, File};
use std::io::Read;
use std::io::Write;
use std::ops::Deref;
use std::thread;
use std::thread::{current, park};
use std::time::{Duration, Instant};

mod fern;
mod grammar;
mod json;
pub mod lexer;
mod parser;

fn main() -> Result<(), Box<dyn Error>> {
    Logger::try_with_str("trace, core::grammar = info")?
        .format(flexi_logger::colored_default_format)
        .start_with_specfile("log.toml")?;
    tbl_driven_lexer()?;
    Ok(())
}

fn tbl_driven_lexer() -> Result<(), Box<dyn Error>> {
    let mut file = fs::File::open("data/grammar/fern.lg").unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    let g = grammar::lg::LexicalGrammar::from(buf.clone());
    let nfa = grammar::lg::StateGraph::from(g.clone());
    let mut f = File::create("nfa.dot").unwrap();
    grammar::lg::render(&nfa, &mut f);
    let dfa = nfa.convert_to_dfa();
    let mut f = File::create("dfa.dot").unwrap();
    grammar::lg::render(&dfa, &mut f);
    let mut table = dfa.build_table();
    buf.clear();

    let mut file = fs::File::open("data/grammar/keywords.lg").unwrap();
    file.read_to_string(&mut buf).unwrap();
    let g = grammar::lg::LexicalGrammar::from(buf.clone());
    let nfa = grammar::lg::StateGraph::from(g.clone());
    let mut f = File::create("nfa.dot").unwrap();
    grammar::lg::render(&nfa, &mut f);
    let dfa = nfa.convert_to_dfa();
    let mut f = File::create("dfa.dot").unwrap();
    grammar::lg::render(&dfa, &mut f);
    let keywords = dfa.build_table();

    let name_token = table.terminal_map.iter().position(|x| x == "NAME").unwrap();
    warn!("{}", name_token);
    table.add_table(name_token, keywords);

    let tokens: LinkedList<(Vec<Token>, Vec<Data>)> = {
        let file = File::open("data/test.fern")?;
        let mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };
        thread::scope(|s| {
            let mut lexer: ParallelLexer<FernLexer> = ParallelLexer::new(table.clone(), s, 1, &[0], 0);
            let batch = lexer.new_batch();
            lexer.add_to_batch(&batch, &mmap[..], 0);
            let tokens = lexer.collect_batch(batch);
            lexer.kill();
            tokens
        })
    };

    info!("{:?}", &tokens);
    for (l, _) in &tokens {
        for t in l {
            info!("{}", table.terminal_map[*t]);
        }
    }

    let mut now = Instant::now();
    let mut raw = RawGrammar::from("data/grammar/fern.g", table.terminal_map)?;
    raw.delete_repeated_rhs()?;
    let grammar = OpGrammar::new(raw)?;
    grammar.to_file("data/grammar/fern-fnf.g");

    let mut now = Instant::now();
    let (tree, time): (FernParseTree, Duration) = {
        let mut parser = ParallelParser::new(grammar.clone(), 1);
        parser.parse(tokens);
        parser.parse(LinkedList::from([(vec![grammar.delim], Vec::new())]));
        let time = parser.time_spent_rule_searching.clone();
        (parser.collect_parse_tree().unwrap(), time)
    };

    tree.print();
    info!("Total Time to parse: {:?}", now.elapsed());
    info!("└─Total Time spent rule-searching: {:?}", time);
    Ok(())
}
