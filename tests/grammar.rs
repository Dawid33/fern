extern crate core;

use std::fs;
use std::io::Read;

// #[test]
// pub fn test_reader_json() {
//     let mut file = fs::File::open("data/grammar/json.g").unwrap();
//     let mut buf = String::new();
//     file.read_to_string(&mut buf).unwrap();
//
//     // TODO: Actually test this here.
//     read_grammar_file(buf.as_str()).unwrap();
// }
//
// #[test]
// pub fn test_reader_lua() {
//     let mut file = fs::File::open("data/grammar/lua.g").unwrap();
//     let mut buf = String::new();
//     file.read_to_string(&mut buf).unwrap();
//
//     // TODO: Actually test this here.
//     read_grammar_file(buf.as_str()).unwrap();
// }
//
// #[test]
// pub fn test_reader_fern() {
//     let config: simplelog::Config = simplelog::ConfigBuilder::new()
//         .set_time_level(simplelog::LevelFilter::Off)
//         .set_target_level(simplelog::LevelFilter::Off)
//         .set_thread_level(simplelog::LevelFilter::Off)
//         .build();
//     let _ = simplelog::SimpleLogger::init(simplelog::LevelFilter::Debug, config);
//
//     let mut file = fs::File::open("data/grammar/fern.g").unwrap();
//     let mut buf = String::new();
//     file.read_to_string(&mut buf).unwrap();
//
//     // TODO: Actually test this here.
//     let _ = read_grammar_file(buf.as_str()).unwrap();
// }
