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

mod parser;
mod lexer;
mod grammar;

fn main() -> Result<(), Box<dyn Error>> {
    Ok(())
}
