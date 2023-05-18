pub extern crate core;
use std::collections::LinkedList;
use std::error::Error;
use std::fs::File;
use std::iter::zip;
use std::thread;
use memmap::MmapOptions;
use tungstenite::accept;
use core::lexer;
use crate::common::test_lex;
use crate::core::fern::{FernLexer, FernLexerState};
use crate::core::grammar::Token;
use crate::core::lexer::ParallelLexer;
use crate::core::lua::LuaLexer;

mod common;


#[test]
fn let_stmt_lex_test() {
   test_lex("tests/data/let_stmt.testfile").unwrap();
}
