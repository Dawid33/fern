use log::{debug, info, trace, warn};
use memmap::MmapOptions;

use crate::fern::{FernLexer, FernParseTree};
use crate::grammar::lg::{self, LexingTable, LookupResult, State, Token};
use crate::grammar::opg::{OpGrammar, RawGrammar};
use crate::lexer::{split_mmap_into_chunks, Data, LexerError, LexerInterface, ParallelLexer};
use crate::parser::{Node, Parser};
use crate::parsetree::ParseTree;
use std::cmp::max;
use std::collections::LinkedList;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::thread;
use std::time::{Duration, Instant};

pub fn compile() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    let lg = Instant::now();
    let first_lg = Instant::now();
    let mut file = File::open("data/grammar/eslang.lg").unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    let g = lg::LexicalGrammar::from(buf.clone());
    let nfa = lg::StateGraph::from(g.clone());
    let mut f = File::create("nfa.dot").unwrap();
    lg::render(&nfa, &mut f);
    let dfa = nfa.convert_to_dfa();
    let mut f = File::create("dfa.dot").unwrap();
    lg::render(&dfa, &mut f);
    let mut table = dfa.build_table();
    table.terminal_map.push("UMINUS".to_string());
    let first_lg = first_lg.elapsed();
    buf.clear();

    let second_lg = Instant::now();
    let mut file = File::open("data/grammar/eslang_keywords.lg").unwrap();
    file.read_to_string(&mut buf).unwrap();
    let g = lg::LexicalGrammar::from(buf.clone());
    let nfa = lg::StateGraph::from(g.clone());
    let mut f = File::create("keyword_nfa.dot").unwrap();
    lg::render(&nfa, &mut f);
    let dfa = nfa.convert_to_dfa();
    let mut f = File::create("keyword_dfa.dot").unwrap();
    lg::render(&dfa, &mut f);
    let keywords = dfa.build_table();
    let second_lg = second_lg.elapsed();

    for x in keywords.table {
        println!("{:?}, {:?}", x.0 as char, x.1);
    }

    // let name_token = table.terminal_map.iter().position(|x| x == "TEXT").unwrap();
    // table.add_table(name_token, keywords);

    // let lex_time = Instant::now();
    // let tokens: LinkedList<(Vec<Token>, Vec<Data>)> = {
    //     let file = File::open("data/test.eslang")?;
    //     let mut mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };
    //     let chunks = split_mmap_into_chunks(&mut mmap, 50000).unwrap();
    //     thread::scope(|s| {
    //         let mut lexer: ParallelLexer<EslangLexer> = ParallelLexer::new(table.clone(), s, 1);
    //         let batch = lexer.new_batch();
    //         for task in chunks.iter().enumerate() {
    //             lexer.add_to_batch(&batch, task.1, task.0);
    //         }
    //         let tokens = lexer.collect_batch(batch);
    //         lexer.kill();
    //         tokens
    //     })
    // };
    // let lex_time = lex_time.elapsed();

    // info!("{:?}", &tokens);
    // for (l, _) in &tokens {
    //     for t in l {
    //         info!("{}", table.terminal_map[*t]);
    //     }
    // }

    // let grammar_time = Instant::now();
    // let mut raw = RawGrammar::from("data/grammar/eslang.g", table.terminal_map.clone())?;
    // raw.delete_repeated_rhs()?;
    // let grammar = OpGrammar::new(raw)?;
    // let grammar_time = grammar_time.elapsed();

    // let parse_time = Instant::now();
    // let tree: ParseTree = {
    //     let mut trees = Vec::new();
    //     for (partial_tokens, partial_data) in tokens {
    //         let mut parser = Parser::new(grammar.clone());
    //         parser.parse(partial_tokens, partial_data);
    //         parser.parse(vec![grammar.delim], Vec::new());
    //         trees.push(parser.collect_parse_tree().unwrap());
    //     }

    //     trees.reverse();
    //     let mut first = trees.pop().unwrap();
    //     while let Some(tree) = trees.pop() {
    //         first.merge(tree);
    //     }
    //     first.into_tree()
    // };
    // let parse_time = parse_time.elapsed();

    // tree.print();
    // let mut f = File::create("ptree.dot").unwrap();
    // tree.dot(&mut f).unwrap();

    // info!("Time to build lexical grammar: {:?}", lg);
    // info!("Time to build second lexical grammar: {:?}", second_lg);
    // info!("Time to lex: {:?}", lex_time);
    // info!("Time to build parsing grammar: {:?}", grammar_time);
    // info!("Time to parse: {:?}", parse_time);
    // info!("Total run time : {:?}", start.elapsed());

    Ok(())
}

pub struct EslangLexer {
    pub table: LexingTable,
    pub start_state: State,
    pub state: State,
    pub buf: String,
    pub tokens: Vec<Token>,
    pub data: Vec<Data>,
    pub whitespace_token: Token,
    had_whitespace: bool,
}

impl LexerInterface for EslangLexer {
    fn new(table: LexingTable, start_state: usize) -> Self {
        let whitespace_token = table.terminal_map.iter().position(|x| x == "WHITESPACE").unwrap();
        Self {
            table,
            whitespace_token,
            had_whitespace: false,
            tokens: Vec::new(),
            start_state,
            buf: String::new(),
            state: start_state,
            data: Vec::new(),
        }
    }
    fn consume(&mut self, input: u8) -> Result<(), LexerError> {
        let mut reconsume = true;
        while reconsume {
            reconsume = false;
            let result = self.table.get(input, self.state);
            match result {
                LookupResult::Terminal(mut t) => {
                    info!("c, t: {}, {}", input as char, self.table.terminal_map[t]);
                    if let Some((table, offset)) = self.table.sub_tables.get(&t) {
                        let mut state = 0;
                        let buf = format!("{}", self.buf);
                        for c in buf.chars() {
                            match table.get(c as u8, state) {
                                LookupResult::Terminal(token) => {
                                    t = token + offset;
                                    break;
                                }
                                LookupResult::State(s) => {
                                    state = s;
                                }
                                LookupResult::Err => break,
                            }
                        }
                        if let Some(token) = table.try_get_terminal(state) {
                            t = token + offset;
                        }
                    }

                    if t != self.whitespace_token {
                        self.tokens.push(t);
                        self.data.push(Data {
                            token_index: self.tokens.len() - 1,
                            raw: self.buf.clone(),
                        });
                        self.had_whitespace = false;
                    } else {
                        self.had_whitespace = true;
                    }
                    self.buf.clear();
                    self.state = 0;
                    reconsume = true;
                }
                LookupResult::State(s) => {
                    self.buf.push(input as char);
                    self.state = s;
                }
                LookupResult::Err => {
                    warn!("Lexing Error when transitioning state. state : {}", self.state);
                }
            }
        }
        return Ok(());
    }
    fn take(self) -> (State, Vec<Token>, Vec<Data>) {
        (self.state, self.tokens, self.data)
    }
}
