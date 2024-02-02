#![allow(dead_code)]
// extern crate core;

// pub mod lexing;
// pub use lexing::test_lex;

// use std::error::Error;
// use std::sync::{Mutex, Once};
// use memmap::Mmap;

// use core::grammar::OpGrammar;
// use core::grammar::RawGrammar;
// use core::grammar::Token;
// use core::parser::ParseTree;
// use crate::common::LexerDataParserState::{AwaitingToken, InToken};

// use crate::common::Parser::{LexerData, NoParser};
// use crate::common::State::{AwaitingWord, InCode, InData, InWord};

// static INIT: Once = Once::new();
// pub static FERN_GRAMMAR: Mutex<Option<OpGrammar>> = Mutex::new(None);

// pub fn get_grammar(g: &Mutex<Option<OpGrammar>>) -> OpGrammar {
//     INIT.call_once(|| {
//         let mut option = g.lock().unwrap();
//         let mut raw = RawGrammar::from("data/grammar/fern.g").unwrap();
//         raw.delete_repeated_rhs().unwrap();
//         let grammar = OpGrammar::new(raw).unwrap();
//         let _ = option.insert(grammar);
//     });
//     let g = g.lock().unwrap().clone().unwrap();
//     return g;
// }

// pub struct TestFile<'a> {
//     pub code: &'a [u8],
//     pub lexer_data: Option<Vec<Token>>,
//     pub parser_data: Option<ParseTree>,
// }

// enum State {
//     AwaitingWord,
//     InWord,
//     InData,
//     InCode,
// }

// enum LexerDataParserState {
//     InToken,
//     AwaitingToken,
// }

// struct LexerDataParser<'a> {
//     tokens: Vec<Token>,
//     buf: String,
//     state: LexerDataParserState,
//     g: &'a OpGrammar,
// }

// impl<'a> LexerDataParser<'a> {
//     pub fn new(g: &'a OpGrammar) -> Self {
//         Self {
//             tokens: Vec::new(),
//             buf: String::new(),
//             state: AwaitingToken,
//             g
//         }
//     }

//     pub fn parse(&mut self, c: char) -> Result<(), Box<dyn Error>> {
//         match self.state {
//             InToken => {
//                 match c {
//                     'a'..='z' | 'A'..='Z' | '_' => self.buf.push(c),
//                     ' ' | '\n' | '\t' => self.build_token(),
//                     _ => (),
//                 }
//             }
//             AwaitingToken => {
//                 match c {
//                     'a'..='z' | 'A'..='Z' | '_' => {
//                         self.state = InToken;
//                         self.buf.push(c)
//                     },
//                     _ => (),
//                 }
//             }
//         }
//         Ok(())
//     }

//     pub fn build(mut self) -> Vec<Token> {
//         if !self.buf.is_empty() {
//             self.build_token();
//         }
//         self.tokens
//     }

//     fn build_token(&mut self) {
//         if !self.buf.is_empty() {
//             let t = self.g.token_reverse.get(self.buf.as_str());
//             if let Some(t) = t {
//                 self.tokens.push(t.0);
//             } else {
//                panic!("Expected data parse error: Lexeme doesn't exist in grammar = {}", self.buf.as_str())
//             }
//             self.buf.clear();
//         }
//     }
// }

// enum Parser<'a> {
//     NoParser,
//     LexerData(LexerDataParser<'a>),
// }

// pub fn read_test_file<'a>(mmap: &'a Mmap, g: &OpGrammar) -> Result<TestFile<'a>, Box<dyn Error>> {
//     let mut buf = String::new();
//     let mut state = AwaitingWord;
//     let mut parser: Parser = NoParser;

//     let mut result = TestFile {
//         code: &mmap[..],
//         lexer_data: None,
//         parser_data: None,
//     };

//     for (i, c) in mmap[..].iter().enumerate() {
//         let c = *c as char;
//         match state {
//             AwaitingWord => {
//                 if c == '`' {
//                     state = InWord;
//                 }
//             }
//             InWord => {
//                 match c {
//                     'a'..='z' | 'A'..='Z' => buf.push(c),
//                     '`' => (),
//                     _ => {
//                         state = InData;
//                         match buf.as_str() {
//                             "lexer" => {
//                                 parser = LexerData(LexerDataParser::new(g));
//                                 buf.clear();
//                             }
//                             "end" => {
//                                 parser = NoParser;
//                                 state = InCode;
//                                 buf.clear();
//                             }
//                             _ => panic!("Unknown type of test data."),
//                         }
//                     }
//                 }
//             }
//             InData => {
//                 if c == '`' {
//                     state = InWord;
//                     match parser {
//                         NoParser => {
//                             panic!("Parsed with no parser, probably a bug.");
//                         }
//                         LexerData(lexer_data_parser) => {
//                             result.lexer_data = Some(lexer_data_parser.build());
//                         }
//                     }
//                     parser = NoParser;
//                 } else {
//                     match &mut parser {
//                         NoParser => panic!("Parsing with no parser, probably a bug."),
//                         LexerData(lexer_data_parser) => {lexer_data_parser.parse(c)?}
//                     }
//                 }
//             }
//             InCode => {
//                 result.code = &mmap[i..];
//                 break;
//             }
//         }
//     }

//     Ok(result)
// }
