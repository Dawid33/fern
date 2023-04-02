extern crate core;

use std::fs;
use core::grammar::reader::read_grammar_file;
use std::io::Read;

#[test]
pub fn test_reader_json() {
    let mut file = fs::File::open("json.g").unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();

    // TODO: Actually test this here.
    read_grammar_file(buf.as_str()).unwrap();
}

#[test]
pub fn test_reader_lua() {
    let mut file = fs::File::open("lua.g").unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();

    // TODO: Actually test this here.
    read_grammar_file(buf.as_str()).unwrap();
}
