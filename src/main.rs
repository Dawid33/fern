#![cfg_attr(debug_assertions, allow(dead_code, unused_imports))]
#![allow(ambiguous_glob_reexports)]
extern crate core;

use crate::fern::FernLexer;
use crate::grammar::opg::{OpGrammar, RawGrammar};
use crate::lexer::{split_mmap_into_chunks, Data, ParallelLexer};
use crate::parser::Parser;
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

mod eslang;
mod fern;
mod grammar;
mod json;
pub mod lexer;
mod parser;
pub mod parsetree;

fn main() -> Result<(), Box<dyn Error>> {
    Logger::try_with_str("trace, core::grammar = info")?
        .format(flexi_logger::colored_default_format)
        .start_with_specfile("log.toml")?;
    fern::compile()?;
    // json::compile()?;
    // eslang::compile()?;
    Ok(())
}
