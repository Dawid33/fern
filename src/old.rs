
// let mut f = File::create("rhs_dict.txt").unwrap();
// for (key, val) in &rhs_dict {
//     let mut builder = String::new();
//     builder.push_str(format!("{} = [", token_raw.get(&key).unwrap()).as_str());
//     let mut val = val.clone();
//     val.sort();
//     if !val.is_empty() {
//         let mut val_iter = val.get(0).unwrap().iter();
//         builder.push_str("[");
//         if val_iter.len() > 0 {
//             builder.push_str(format!("\'{}\'", token_raw.get(val_iter.next().unwrap()).unwrap()).as_str());
//         }
//         while let Some(t) = val_iter.next() {
//             builder.push_str(", ");
//             builder.push_str(format!("\'{}\'", token_raw.get(t).unwrap()).as_str());
//         }
//         builder.push_str("]");
//         if val.len() > 1 {
//             for k in &val[1..val.len()] {
//                 builder.push_str(", [");
//                 let mut val_iter = k.iter();
//                 if val_iter.len() > 0 {
//                     builder.push_str(format!("\'{}\'", token_raw.get(val_iter.next().unwrap()).unwrap()).as_str());
//                 }
//                 while let Some(t) = val_iter.next() {
//                     builder.push_str(", ");
//                     builder.push_str(format!("\'{}\'", token_raw.get(t).unwrap()).as_str());
//                 }
//                 builder.push_str("]");
//             }
//         }
//     }
//     builder.push_str("]\n");
//     f.write(builder.as_bytes());
// }


// let mut f = File::create("new_dict_rules.txt").unwrap();
// for (key, val) in &new_dict_rules {
//     let mut builder = String::new();
//     builder.push_str("[");
//     if !key.is_empty() {
//         let mut val_iter = key.get(0).unwrap().iter();
//         if val_iter.len() > 0 {
//             builder.push_str(format!("\'{}\'", token_raw.get(val_iter.next().unwrap()).unwrap()).as_str());
//         }
//         while let Some(t) = val_iter.next() {
//             builder.push_str(", ");
//             builder.push_str(format!("\'{}\'", token_raw.get(t).unwrap()).as_str());
//         }
//         if key.len() > 1 {
//             for k in &key[1..key.len()] {
//                 builder.push_str(", [");
//                 let mut val_iter = key.get(0).unwrap().iter();
//                 if val_iter.len() > 0 {
//                     builder.push_str(format!("\'{}\'", token_raw.get(val_iter.next().unwrap()).unwrap()).as_str());
//                 }
//                 while let Some(t) = val_iter.next() {
//                     builder.push_str(", ");
//                     builder.push_str(format!("\'{}\'", token_raw.get(t).unwrap()).as_str());
//                 }
//             }
//         }
//         builder.push_str("] = [");
//
//         let mut sorted = Vec::new();
//         for x in val.iter() {
//             sorted.push(token_raw.get(x).unwrap());
//         }
//         sorted.sort();
//
//         let mut val_iter = sorted.iter();
//         if val_iter.len() > 0 {
//             builder.push_str(format!("\'{}\'", val_iter.next().unwrap()).as_str());
//         }
//         while let Some(t) = val_iter.next() {
//             builder.push_str(", ");
//             builder.push_str(format!("\'{}\'", t).as_str());
//         }
//     }
//     builder.push_str("]\n");
//     f.write(builder.as_bytes());
// }

// let mut f = File::create("finalforreal.txt").unwrap();
// for (key, val) in &new_dict_rules{
//     let mut builder = String::new();
//     builder.push_str("[");
//     if !key.is_empty() {
//         builder.push_str("[");
//         into_str(&mut builder, key.get(0).unwrap());
//         builder.push_str("]");
//         if key.len() > 1 {
//             for k in &key[1..key.len()] {
//                 builder.push_str(", [");
//                 into_str(&mut builder, k);
//                 builder.push_str("]");
//             }
//         } else {
//             // builder.push_str(",");
//         }
//         builder.push_str("] = [");
//
//         into_str(&mut builder, &val.clone().into_iter().collect());
//     }
//     builder.push_str("]\n");
//     f.write(builder.as_bytes());
// }

// Print rules to file
// let mut f = File::create("rules.txt").unwrap();
// for r in &rules {
//     let mut line = String::from(format!("{} : [", token_raw.get(&r.left).unwrap()));
//     let mut iter = r.right.iter();
//     if let Some(x) = iter.next() {
//         line.push_str(format!("'{}'", token_raw.get(x).unwrap()).as_str());
//     }
//     while let Some(x) = iter.next() {
//         line.push_str(format!(", '{}'", token_raw.get(x).unwrap()).as_str());
//     }
//     line.push_str("]\n");
//     f.write(line.as_bytes());
// }


// let mut f = File::create("V.txt").unwrap();
// for val in &v {
//     let mut builder = String::new();
//     builder.push_str("[");
//     let mut sorted = Vec::new();
//     for x in val.iter() {
//         sorted.push(token_raw.get(x).unwrap());
//     }
//     sorted.sort();
//
//     let mut val_iter = sorted.iter();
//     if val_iter.len() > 0 {
//         builder.push_str(format!("\'{}\'", val_iter.next().unwrap()).as_str());
//     }
//     while let Some(t) = val_iter.next() {
//         builder.push_str(", ");
//         builder.push_str(format!("\'{}\'", t).as_str());
//     }
//     builder.push_str("]\n");
//     f.write(builder.as_bytes());
// }
// let mut f = File::create("debug.txt").unwrap();
// for (key, val) in &new_dict_rules{
//     let mut builder = String::new();
//     builder.push_str("[");
//     if !key.is_empty() {
//         builder.push_str("[");
//         into_str(&mut builder, key.get(0).unwrap());
//         builder.push_str("]");
//         if key.len() > 1 {
//             for k in &key[1..key.len()] {
//                 builder.push_str(", [");
//                 into_str(&mut builder, k);
//                 builder.push_str("]");
//             }
//         } else {
//             // builder.push_str(",");
//         }
//         builder.push_str("] = [");
//
//         into_str(&mut builder, &val.clone().into_iter().collect());
//     }
//     builder.push_str("]\n");
//     f.write(builder.as_bytes());
// }

// let into_str = |builder: &mut String, input: &Vec<Token>| {
//     let mut output = Vec::new();
//     let mut should_sort = true;
//     for x in input {
//         output.push(format!("{}", token_raw.get(x).unwrap()));
//         if terminals.contains(x) {
//             should_sort = false;
//         }
//     }
//     if should_sort {
//         output.sort();
//     }
//     let mut iter = output.into_iter();
//     if let Some(x) = iter.next() {
//         builder.push_str(format!("\'{}\'", x).as_str());
//     }
//     while let Some(x) = iter.next() {
//         builder.push_str(format!(", \'{}\'", x).as_str());
//     }
// };

// extern crate core;
//
// use core::grammar::OpGrammar;
// use core::lex_lua;
// use log::info;
// use std::error::Error;
// use std::fs::File;
//
// fn test_lua(input: &str, expected: Vec<&str>) {
//     let g = OpGrammar::from("data/grammar/lua-fnf.g");
//     let result = lex_lua(input, &g).unwrap();
//     let mut size = 0;
//     for list in result {
//         size += list.len();
//         for (i, t) in list.iter().enumerate() {
//             assert_eq!(
//                 *t,
//                 g.token_reverse.get(*expected.get(i).unwrap()).unwrap().0,
//                 "Recieved {}, expected {}.",
//                 g.token_raw.get(t).unwrap(),
//                 expected.get(i).unwrap()
//             );
//         }
//     }
//     assert_eq!(
//         size,
//         expected.len(),
//         "Number of recieved tokens ({}) doesn't equal number of expected tokens ({}).",
//         size,
//         expected.len()
//     );
// }

// #[test]
// fn test_simple_stmt() {
//     test_lua("local x = 0;", vec!["LOCAL", "NAME", "XEQ", "NUMBER", "SEMI"]);
// }

// #[test]
// fn test_for() {
//     test_lua(
//         "for c = 0, 323 do R[c] = {} end",
//         vec![
//             "FOR", "NAME", "XEQ", "NUMBER", "COMMA", "NUMBER", "DO", "NAME", "LBRACK", "NAME", "RBRACK", "XEQ",
//             "LBRACE", "RBRACE", "END",
//         ],
//     );
// }

// #[test]
// fn full_test() -> Result<(), Box<dyn Error>> {
//     Logger::try_with_str("trace, core::grammar = info")?;
//     let mut now = Instant::now();
//     let grammar = OpGrammar::from("data/grammar/json.g");
//     info!("Total Time to generate grammar : {:?}", now.elapsed());
//     now = Instant::now();
//
//     let mut tokens: LinkedList<Vec<Token>> = LinkedList::new();
//     {
//         let file = File::open("data/test.json")?;
//         let mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };
//         info!("Total time to load file: {:?}", now.elapsed());
//         now = Instant::now();
//         thread::scope(|s| {
//             let mut lexer: ParallelLexer<JsonLexerState, JsonLexer> = ParallelLexer::new(
//                 grammar.clone(),
//                 s,
//                 1,
//                 &[JsonLexerState::Start, JsonLexerState::InString],
//                 JsonLexerState::Start,
//             );
//             let batch = lexer.new_batch();
//             lexer.add_to_batch(&batch, &mmap[..], 0);
//             tokens = lexer.collect_batch(batch);
//             lexer.kill();
//         });
//     }
//     info!("Total Lexing Time: {:?}", now.elapsed());
//
//     let tree: ParseTree = {
//         now = Instant::now();
//         let mut parser = ParallelParser::new(grammar.clone(), 1);
//         parser.parse(tokens);
//         parser.parse(LinkedList::from([vec![grammar.delim]]));
//         parser.collect_parse_tree().unwrap()
//     };
//
//     debug!("Total Parsing Time: {:?}", now.elapsed());
//
//     tree.print();
//
//     now = Instant::now();
//
//     debug!("Total Time For ParseTree -> AST Conversion: {:?}", now.elapsed());
//     Ok(())
// }

// #[test]
// fn full_test_parallel() -> Result<(), Box<dyn Error>> {
//     Logger::try_with_str("trace, core::grammar = info")?;
//
//     let now = Instant::now();
//     let grammar = OpGrammar::from("data/grammar/json.g");
//     info!("Total Time to generate grammar : {:?}", now.elapsed());
//     let now = Instant::now();
//
//     let file = File::open("data/json/10KB.json").unwrap();
//     let mut memmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
//     info!("Total time to load file: {:?}", now.elapsed());
//     let mut now = Instant::now();
//
//     let chunks = core::lexer::split_mmap_into_chunks(&mut memmap, 6000).unwrap();
//
//     let tokens = thread::scope(|s| {
//         let mut lexer: ParallelLexer<JsonLexerState, JsonLexer> = ParallelLexer::new(
//             grammar.clone(),
//             s,
//             1,
//             &[JsonLexerState::Start, JsonLexerState::InString],
//             JsonLexerState::Start,
//         );
//         let batch = lexer.new_batch();
//         for task in chunks.iter().enumerate() {
//             lexer.add_to_batch(&batch, task.1, task.0);
//         }
//         let output = lexer.collect_batch(batch);
//         lexer.kill();
//         output
//     });
//
//     info!("Total Lexing Time: {:?}", now.elapsed());
//
//     // let _: ParseTree = {
//     //     now = Instant::now();
//     //     let mut parser = ParallelParser::new(grammar.clone(), 1);
//     //     parser.parse(tokens);
//     //     parser.parse(LinkedList::from([vec![grammar.delim]]));
//     //     parser.collect_parse_tree().unwrap()
//     // };
//     //
//     // info!("Total Parsing Time: {:?}", now.elapsed());
//     Ok(())
// }

// #[test]
// fn full_test() -> Result<(), Box<dyn Error>> {
//     Logger::try_with_str("trace, core::grammar = info")?;
//     let mut now = Instant::now();
//     let grammar = OpGrammar::from("data/grammar/json.g");
//     info!("Total Time to generate grammar : {:?}", now.elapsed());
//     now = Instant::now();
//
//     let mut tokens: LinkedList<Vec<Token>> = LinkedList::new();
//     {
//         let file = File::open("data/test.json")?;
//         let mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };
//         info!("Total time to load file: {:?}", now.elapsed());
//         now = Instant::now();
//         thread::scope(|s| {
//             let mut lexer: ParallelLexer<JsonLexerState, JsonLexer> = ParallelLexer::new(
//                 grammar.clone(),
//                 s,
//                 1,
//                 &[JsonLexerState::Start, JsonLexerState::InString],
//                 JsonLexerState::Start,
//             );
//             let batch = lexer.new_batch();
//             lexer.add_to_batch(&batch, &mmap[..], 0);
//             tokens = lexer.collect_batch(batch);
//             lexer.kill();
//         });
//     }
//     info!("Total Lexing Time: {:?}", now.elapsed());
//
//     let tree: ParseTree = {
//         now = Instant::now();
//         let mut parser = ParallelParser::new(grammar.clone(), 1);
//         parser.parse(tokens);
//         parser.parse(LinkedList::from([vec![grammar.delim]]));
//         parser.collect_parse_tree().unwrap()
//     };
//
//     debug!("Total Parsing Time: {:?}", now.elapsed());
//
//     tree.print();
//
//     now = Instant::now();
//
//     debug!("Total Time For ParseTree -> AST Conversion: {:?}", now.elapsed());
//     Ok(())
// }

// #[test]
// fn full_test_parallel() -> Result<(), Box<dyn Error>> {
//     Logger::try_with_str("trace, core::grammar = info")?;
//
//     let now = Instant::now();
//     let grammar = OpGrammar::from("data/grammar/json.g");
//     info!("Total Time to generate grammar : {:?}", now.elapsed());
//     let now = Instant::now();
//
//     let file = File::open("data/json/10KB.json").unwrap();
//     let mut memmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
//     info!("Total time to load file: {:?}", now.elapsed());
//     let mut now = Instant::now();
//
//     let chunks = core::lexer::split_mmap_into_chunks(&mut memmap, 6000).unwrap();
//
//     let tokens = thread::scope(|s| {
//         let mut lexer: ParallelLexer<JsonLexerState, JsonLexer> = ParallelLexer::new(
//             grammar.clone(),
//             s,
//             1,
//             &[JsonLexerState::Start, JsonLexerState::InString],
//             JsonLexerState::Start,
//         );
//         let batch = lexer.new_batch();
//         for task in chunks.iter().enumerate() {
//             lexer.add_to_batch(&batch, task.1, task.0);
//         }
//         let output = lexer.collect_batch(batch);
//         lexer.kill();
//         output
//     });
//
//     info!("Total Lexing Time: {:?}", now.elapsed());
//
//     // let _: ParseTree = {
//     //     now = Instant::now();
//     //     let mut parser = ParallelParser::new(grammar.clone(), 1);
//     //     parser.parse(tokens);
//     //     parser.parse(LinkedList::from([vec![grammar.delim]]));
//     //     parser.collect_parse_tree().unwrap()
//     // };
//     //
//     // info!("Total Parsing Time: {:?}", now.elapsed());
//     Ok(())
// }
