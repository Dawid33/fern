// use std::error::Error;
// use std::fs::File;
// use std::iter::zip;
// use std::thread;
// use memmap::MmapOptions;
// use crate::common;
// use crate::core::fern::{FernLexer, FernLexerState};
// use crate::core::grammar::Token;
// use crate::core::lexer::ParallelLexer;

// pub fn test_lex(path: &str) -> Result<(), Box<dyn Error>>{
//     let g = common::get_grammar(&common::FERN_GRAMMAR);
//     let file = File::open(path).unwrap();
//     let mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
//     let data = common::read_test_file(&mmap, &g).unwrap();

//     let tokens: Vec<Token> = {
//         thread::scope(|s| {
//             let mut lexer: ParallelLexer<FernLexerState, FernLexer> =
//                 ParallelLexer::new(&g, s, 1, &[FernLexerState::Start], FernLexerState::Start);
//             let batch = lexer.new_batch();
//             lexer.add_to_batch(&batch, data.code, 0);
//             let tokens = lexer.collect_batch(batch);
//             lexer.kill();
//             let mut result = Vec::new();
//             for mut x in tokens {
//                 result.append(&mut x);
//             }
//             result
//         })
//     };

//     if let Some(expected) = data.lexer_data {
//         let zipped = zip(tokens.iter(), expected.iter());
//         for (actual, expected) in zipped {
//             assert_eq!(*expected, *actual);
//         }
//         assert_eq!(expected.len(), tokens.len(), "Number of expected tokens differs from actual: expected : {}, actual: {}", tokens.len(), expected.len());
//     } else {
//         panic!("No expected lexer data for this test file.");
//     }
//     Ok(())
// }
