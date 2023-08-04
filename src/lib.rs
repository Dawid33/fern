#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]

use crate::grammar::{OpGrammar, Token};
use crate::lexer::lua::{LuaLexer, LuaLexerState};
use crate::lexer::ParallelLexer;
use memmap::MmapOptions;
use std::collections::LinkedList;
use std::error::Error;
use std::fs::File;
use std::thread;

pub mod analysis;
pub mod cfg;
pub mod grammar;
pub mod ir;
pub mod lexer;
pub mod parser;
pub mod server;
pub mod slab;

use crate::lexer::fern::{FernLexer, FernLexerState};
pub use grammar::*;
pub use parser::*;

// pub fn lex_fern(input: &str, grammar: &OpGrammar) -> Result<(), Box<dyn Error>> {
//     let _: LinkedList<Vec<Token>> = {
//         thread::scope(|s| {
//             let mut lexer: ParallelLexer<FernLexerState, FernLexer> =
//                 ParallelLexer::new(&grammar, s, 1, &[FernLexerState::Start], FernLexerState::Start);
//             let batch = lexer.new_batch();
//             lexer.add_to_batch(&batch, &input.as_bytes()[..], 0);
//             let tokens = lexer.collect_batch(batch);
//             lexer.kill();
//             tokens
//         })
//     };
//     Ok(())
// }
