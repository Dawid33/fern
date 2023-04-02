#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
extern crate core;

use core::grammar::Grammar;
use core::lexer::json::JsonToken;
use core::lexer::ParallelLexer;
use core::parser::{ParallelParser, ParseTree};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    Ok(())
}
