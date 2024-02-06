use log::{info, trace};
use memmap::MmapOptions;

use crate::fern::{FernLexer, FernParseTree};
use crate::grammar::lg;
use crate::grammar::opg::{OpGrammar, RawGrammar};
use crate::lexer::{split_mmap_into_chunks, Data, ParallelLexer, Token};
use crate::parser::{Node, ParallelParser, ParseTree};
use std::cmp::max;
use std::collections::LinkedList;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::thread;
use std::time::{Duration, Instant};

fn json() -> Result<(), Box<dyn Error>> {
    let mut file = File::open("data/grammar/json.lg").unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    let g = lg::LexicalGrammar::from(buf.clone());
    let nfa = lg::StateGraph::from(g.clone());
    let mut f = File::create("nfa.dot").unwrap();
    lg::render(&nfa, &mut f);
    let dfa = nfa.convert_to_dfa();
    let mut f = File::create("dfa.dot").unwrap();
    lg::render(&dfa, &mut f);
    let table = dfa.build_table();

    let tokens: LinkedList<(Vec<Token>, Vec<Data>)> = {
        let file = File::open("data/test.json")?;
        let mut mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };
        let chunks = split_mmap_into_chunks(&mut mmap, 5).unwrap();
        thread::scope(|s| {
            let mut lexer: ParallelLexer<FernLexer> = ParallelLexer::new(table.clone(), s, 2, &[0], 0);
            let batch = lexer.new_batch();
            for task in chunks.iter().enumerate() {
                lexer.add_to_batch(&batch, task.1, task.0);
            }
            let tokens = lexer.collect_batch(batch);
            lexer.kill();
            tokens
        })
    };

    trace!("{:?}", &tokens);
    for (l, _) in &tokens {
        for t in l {
            trace!("{}", table.terminal_map[*t]);
        }
    }

    let mut now = Instant::now();
    let mut raw = RawGrammar::from("data/grammar/json.g", table.terminal_map)?;
    raw.delete_repeated_rhs()?;
    let grammar = OpGrammar::new(raw)?;
    grammar.to_file("data/grammar/json-fnf.g");

    let mut now = Instant::now();
    let (tree, time): (ParseTree, Duration) = {
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

#[derive(Debug, Clone)]
pub enum JsonValue {
    Null,
    String(String),
    Number(Number),
    Boolean(bool),
    Object(Object),
    Array(Vec<JsonValue>),
}

#[derive(Debug, Clone)]
pub struct Number {}

#[derive(Debug, Clone)]
pub struct Object {}
