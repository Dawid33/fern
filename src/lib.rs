#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

#[macro_use]
extern crate json as json_parse;

use lexer::LexerInterface;
use std::collections::LinkedList;
use std::error::Error;
use std::fs::File;
use std::io::BufWriter;
use std::thread;
use std::time::{Duration, Instant};
extern crate console_error_panic_hook;

pub mod fern;
pub mod grammar;
pub mod lexer;
pub mod parser;
pub mod parsetree;

use grammar::lg;
use log::{debug, info};

use wasm_bindgen::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
pub fn split_file_into_chunks<'a>(mmap: &'a memmap::Mmap, step: usize) -> Result<Vec<&'a [u8]>, Box<dyn Error>> {
    let mut indices = vec![];
    let mut i = 0;
    let mut prev = 0;

    if mmap.len() < step {
        return Ok(vec![&mmap[..]]);
    }

    while i < mmap.len() {
        if mmap[i] as char != ' ' && mmap[i] as char != '\n' {
            i += 1;
        } else {
            if i + 1 <= mmap.len() {
                i += 1;
            }
            indices.push((prev, i));
            prev = i;
            i += step;
        }
    }
    if prev < mmap.len() {
        indices.push((prev, mmap.len()));
    }

    let mut units = vec![];
    for i in indices {
        units.push(&mmap[i.0..i.1]);
    }
    return Ok(units);
}

#[cfg(not(target_arch = "wasm32"))]
mod json;

#[cfg(target_arch = "wasm32")]
const COMP_TIME_GRAMMAR: &'static str = include_str!("../data/grammar/fern.g");
const COMP_TIME_LEXICAL_GRAMMAR: &'static str = include_str!("../data/grammar/fern.lg");
const COMP_TIME_KEYWORD_LEXICAL_GRAMMAR: &'static str = include_str!("../data/grammar/keywords.lg");

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["document"])]
    fn js_log(a: &str);
}

#[cfg(target_arch = "wasm32")]
use log::{Level, Log, Metadata, Record, SetLoggerError};

#[cfg(target_arch = "wasm32")]
static LOGGER: WebConsoleLogger = WebConsoleLogger {};

#[cfg(target_arch = "wasm32")]
struct WebConsoleLogger {}

#[cfg(target_arch = "wasm32")]
impl Log for WebConsoleLogger {
    #[inline]
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        log(record);
    }

    fn flush(&self) {}
}

/// Print a `log::Record` to the browser's console at the appropriate level.
///
/// This function is useful for integrating with the [`fern`](https://crates.io/crates/fern) logger
/// crate.
///
/// ## Example
/// ```rust,ignore
/// fern::Dispatch::new()
///     .chain(fern::Output::call(console_log::log))
///     .apply()?;
/// ```
#[cfg(target_arch = "wasm32")]
#[cfg_attr(not(feature = "color"), inline)]
pub fn log(record: &Record) {
    #[cfg(not(feature = "color"))]
    {
        // pick the console.log() variant for the appropriate logging level
        js_log(&format!("{}", record.args()));
    }
}

/// Initializes the global logger setting `max_log_level` to the given value.
///
/// ## Example
///
/// ```
/// use log::Level;
/// fn main() {
///     console_log::init_with_level(Level::Debug).expect("error initializing logger");
/// }
/// ```
#[cfg(target_arch = "wasm32")]
#[inline]
pub fn init_with_level(level: Level) -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER)?;
    log::set_max_level(level.to_level_filter());
    Ok(())
}

/// Initializes the global logger with `max_log_level` set to `Level::Info` (a sensible default).
///
/// ## Example
///
/// ```
/// fn main() {
///     console_log::init().expect("error initializing logger");
/// }
/// ```
#[cfg(target_arch = "wasm32")]
#[inline]
pub fn init() -> Result<(), SetLoggerError> {
    init_with_level(Level::Info)
}

#[wasm_bindgen]
#[cfg(target_arch = "wasm32")]
pub fn compile_fern(input: &str) -> String {
    init_with_level(Level::Trace);
    console_error_panic_hook::set_once();

    let g = grammar::lg::LexicalGrammar::from(COMP_TIME_LEXICAL_GRAMMAR);
    let nfa = grammar::lg::StateGraph::from(g.clone());
    let dfa = nfa.convert_to_dfa();
    let mut table = dfa.build_table();
    table.terminal_map.push("UMINUS".to_string());

    let g = grammar::lg::LexicalGrammar::from(COMP_TIME_KEYWORD_LEXICAL_GRAMMAR);
    let nfa = grammar::lg::StateGraph::from(g.clone());
    let dfa = nfa.convert_to_dfa();
    let keywords = dfa.build_table();

    let name_token = table.terminal_map.iter().position(|x| x == "NAME").unwrap();
    table.add_table(name_token, keywords);

    let mut raw = grammar::opg::RawGrammar::new(COMP_TIME_GRAMMAR, table.terminal_map.clone()).unwrap();
    raw.delete_repeated_rhs().unwrap();
    let grammar = grammar::opg::OpGrammar::new(raw).unwrap();

    let mut lexer: fern::FernLexer = fern::FernLexer::new(table.clone(), 0);
    for c in input.chars() {
        lexer.consume(c as u8).unwrap();
    }
    let (_, tokens, data) = lexer.take();

    let tree: parsetree::ParseTree = {
        let mut trees = Vec::new();
        let mut parser = parser::Parser::new(grammar.clone());
        parser.parse(tokens.clone(), data);
        parser.parse(vec![grammar.delim], Vec::new());
        trees.push(parser.collect_parse_tree().unwrap());

        trees.reverse();
        let mut first = trees.pop().unwrap();
        while let Some(tree) = trees.pop() {
            first.merge(tree);
        }
        first.into_tree()
    };

    tree.print();
    let mut result = BufWriter::new(Vec::new());
    tree.dot(&mut result).unwrap();
    let bytes = result.into_inner().unwrap();
    let tree_string = String::from_utf8(bytes).unwrap();

    let ast: fern::FernAst = tree.into();
    ast.print();
    let analysis_output = ast.analysis();

    let mut result = BufWriter::new(Vec::new());
    ast.dot(&mut result).unwrap();
    let bytes = result.into_inner().unwrap();
    let ast_string = String::from_utf8(bytes).unwrap();

    let mut tokens_string = String::from("<code>");
    for t in tokens {
        tokens_string.push_str(format!("<p>{}</p>", table.terminal_map.get(t).unwrap()).as_str());
    }
    tokens_string.push_str("</code>");
    let output = object! {
        tokens: tokens_string,
        ptree: tree_string,
        ast: ast_string,
        analysis: analysis_output
    };
    info!("{}", output);

    return output.to_string();
}
