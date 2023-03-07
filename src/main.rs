use std::error::Error;
use std::fs;
use env_logger;
use log::{debug, error, log_enabled, info, warn, Level};
use env_logger::Env;

mod lexer;
mod error;
use lexer::*;

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let contents = fs::read_to_string("json/1GB.json").expect("Cannot open test file.");
    let mut lexer : JsonLexer = JsonLexer::new();
    
    for c in contents.chars() {
        lexer.consume(c).unwrap();
    }

    info!("Lexer tokens size: {}KB", std::mem::size_of_val(&*lexer.tokens.as_slice()) / 1000);
}
