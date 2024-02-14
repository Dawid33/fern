use log::{info, trace, warn};
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
    let mut file = File::open("data/grammar/json.lg").unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    let g = lg::LexicalGrammar::from(buf.clone());
    let nfa = lg::StateGraph::from(g.clone());
    let mut f = File::create("nfa.dot").unwrap();
    lg::render(&nfa, &mut f);
    let dfa = nfa.convert_to_dfa();
    let mut f = File::create("dfa.dot").unwrap();
    lg::render(&dfa, &mut f);
    let table = dfa.build_table();
    let lg = lg.elapsed();

    let lex_time = Instant::now();
    let tokens: LinkedList<(Vec<Token>, Vec<Data>)> = {
        let file = File::open("data/test.json")?;
        let mut mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };
        let chunks = split_mmap_into_chunks(&mut mmap, 5).unwrap();
        thread::scope(|s| {
            let mut lexer: ParallelLexer<JsonLexer> = ParallelLexer::new(table.clone(), s, 4);
            let batch = lexer.new_batch();
            for task in chunks.iter().enumerate() {
                lexer.add_to_batch(&batch, task.1, task.0);
            }
            let tokens = lexer.collect_batch(batch);
            lexer.kill();
            tokens
        })
    };
    let lex_time = lex_time.elapsed();

    trace!("{:?}", &tokens);
    for (l, _) in &tokens {
        for t in l {
            trace!("{}", table.terminal_map[*t]);
        }
    }

    let grammar_time = Instant::now();
    let mut raw = RawGrammar::from("data/grammar/json.g", table.terminal_map.clone())?;
    raw.delete_repeated_rhs()?;
    let grammar = OpGrammar::new(raw)?;
    let grammar_time = grammar_time.elapsed();
    grammar.to_file("data/grammar/json-fnf.g");

    // let parse_time = Instant::now();
    // let (tree, time): (ParseTree, Duration) = {
    //     let mut parser = Parser::new(grammar.clone(), 1);
    //     parser.parse(tokens);
    //     parser.parse(LinkedList::from([(vec![grammar.delim], Vec::new())]));
    //     let time = parser.time_spent_rule_searching.clone();
    //     (parser.collect_parse_tree().unwrap(), time)
    // };
    // let parse_time = parse_time.elapsed();

    // tree.print();
    info!("Time to build lexical grammar: {:?}", lg);
    info!("Time to lex: {:?}", lex_time);
    info!("Time to build parsing grammar: {:?}", grammar_time);
    // info!("Time to parse: {:?}", parse_time);
    // info!("└─Time spent rule-searching: {:?}", time);
    info!("Total run time : {:?}", start.elapsed());

    Ok(())
}

pub struct JsonLexer {
    pub table: LexingTable,
    pub start_state: State,
    pub state: State,
    pub buf: String,
    pub tokens: Vec<Token>,
    pub data: Vec<Data>,
    pub whitespace_token: Token,
    had_whitespace: bool,
}

impl LexerInterface for JsonLexer {
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
                    // info!("c, t: {}, {}", input as char, self.table.terminal_map[t]);
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
                    // warn!("Lexing Error when transitioning state. state : {}", self.state);
                }
            }
        }
        return Ok(());
    }
    fn take(self) -> (State, Vec<Token>, Vec<Data>) {
        (self.state, self.tokens, self.data)
    }
}
