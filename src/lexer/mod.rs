use crossbeam::sync::Parker;
use crossbeam::sync::Unparker;
use crossbeam_deque::{Injector, Worker};
use crossbeam_skiplist::SkipMap;
use log::info;
use log::trace;
use log::warn;
use std::collections::{HashMap, LinkedList};
use std::error::Error;
use std::fmt::Debug;
use std::fs::File;
use std::hash::Hash;
use std::io::{stdout, Write};
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::task::ready;
use std::thread::{Scope, ScopedJoinHandle};
use std::time::{Duration, Instant};
use std::{iter, thread};
use tinyrand::{RandRange, StdRand};

pub mod error;
// pub mod fern;
// pub mod json;
// pub mod lua;

use crate::grammar::lexical_grammar::LexicalGrammar;
use crate::grammar::lexical_grammar::LexingTable;
use crate::grammar::lexical_grammar::LookupResult;
use crate::grammar::OpGrammar;
use crate::lexer::error::LexerError;
// use crate::lexer::json::{JsonData, JsonLexer, JsonLexerState, JsonTokens};
// use crate::lexer::lua::{LuaLexer, LuaLexerState};
use crossbeam_queue::SegQueue;

type State = usize;
type Token = usize;

#[derive(Debug, Clone)]
pub enum Data {
    NoData,
    String(String),
}

pub struct FernLexer {
    pub table: LexingTable,
    start_state: usize,
    state: usize,
    buf: String,
    tokens: Vec<usize>,
}

impl LexerInterface for FernLexer {
    fn new(table: LexingTable, start_state: usize) -> Self {
        Self {
            table,
            tokens: Vec::new(),
            start_state,
            buf: String::new(),
            state: start_state,
        }
    }
    fn consume(&mut self, input: u8) -> Result<(), LexerError> {
        let mut reconsume = true;
        while reconsume {
            reconsume = false;
            if input.is_ascii_whitespace() && !self.buf.is_empty() {
                if let Some(t) = self.table.try_get_terminal(self.state) {
                    self.tokens.push(t);
                    self.buf.clear();
                    self.state = self.start_state;
                }
            } else if input.is_ascii_whitespace() {
                break;
            } else {
                match self.table.get(input, self.state) {
                    LookupResult::Terminal(t) => {
                        self.tokens.push(t);
                        self.buf.clear();
                        self.state = self.start_state;
                        reconsume = true;
                    }
                    LookupResult::State(s) => {
                        self.buf.push(input as char);
                        self.state = s;
                    }
                    LookupResult::Err => {
                        panic!("Lexing Error when transitioning state. state : {}", self.state);
                    }
                }
            }
        }
        return Ok(());
    }
    fn take(self) -> (usize, Vec<usize>) {
        (self.state, self.tokens)
    }
}

pub struct LexerOutput {
    lists: Option<HashMap<usize, LexerPartialOutput>>,
}

#[allow(unused)]
pub struct LexerPartialOutput {
    list: Vec<usize>,
    finish_state: usize,
    success: bool,
}

pub struct WorkUnit<'a>(usize, &'a [u8], Arc<SkipMap<usize, RwLock<LexerOutput>>>);

pub struct ParallelLexer<'a, Lexer> {
    handles: Vec<(ScopedJoinHandle<'a, ()>, Unparker)>,
    connection: crossbeam_channel::Sender<bool>,
    new_queue: Arc<SegQueue<WorkUnit<'a>>>,
    outputs: HashMap<String, Batch>,
    initial_state: usize,
    _phantom_data: PhantomData<Lexer>,
}

pub struct Batch {
    output: Arc<SkipMap<usize, RwLock<LexerOutput>>>,
    size: usize,
}

pub trait LexerInterface {
    fn new(table: LexingTable, start_state: usize) -> Self;
    fn consume(&mut self, c: u8) -> Result<(), LexerError>;
    fn take(self) -> (usize, Vec<usize>);
}

impl<'a, Lexer> ParallelLexer<'a, Lexer>
where
    Lexer: LexerInterface,
{
    pub fn new(grammar: LexingTable, scope: &'a Scope<'a, '_>, threads: usize, possible_start_states: &[usize], initial_state: usize) -> Self {
        let new_queue: Arc<SegQueue<WorkUnit>> = Arc::new(SegQueue::new());
        let (send, recv) = crossbeam_channel::bounded(threads);
        let outputs: HashMap<String, Batch> = HashMap::new();

        let mut handles = vec![];
        for _ in 0..threads {
            let reciever = recv.clone();
            let new_queue = new_queue.clone();
            let grammar = grammar.clone();
            let start_states: Vec<usize> = Vec::from(possible_start_states);
            let parker = Parker::new();
            let unparker = parker.unparker().clone();

            info!("Before spawning thread");
            handles.push((
                scope.spawn(move || {
                    info!("Inside thread");
                    let mut should_run = true;
                    while should_run {
                        let task = new_queue.pop();
                        if let Some(task) = task {
                            let mut lexers: Vec<(Lexer, usize, bool)> = Vec::new();
                            for state in &start_states {
                                lexers.push((Lexer::new(grammar.clone(), *state), *state, true));
                            }

                            for c in task.1 {
                                for (lexer, _, is_successful) in &mut lexers {
                                    if *is_successful {
                                        if let Err(_) = lexer.consume(*c) {
                                            *is_successful = false;
                                        }
                                    }
                                }
                            }

                            // Send whitespace to make sure any tokens at then end of the chunk
                            // so that any remaining tokens in the buffer actually get created.
                            // This is okay because we split up the input string on word boundaries so
                            // adding whitespace should make no difference.
                            for (lexer, _, is_successful) in &mut lexers {
                                if *is_successful {
                                    if let Err(_) = lexer.consume(' ' as u8) {
                                        *is_successful = false;
                                    }
                                }
                            }

                            let mut map: HashMap<usize, LexerPartialOutput> = HashMap::new();
                            for (lexer, start_state, is_successful) in lexers {
                                let (finish_state, tokens) = lexer.take();
                                map.insert(
                                    start_state,
                                    LexerPartialOutput {
                                        success: is_successful,
                                        finish_state,
                                        list: tokens,
                                    },
                                );
                            }
                            task.2.insert(task.0, RwLock::new(LexerOutput { lists: Some(map) }));
                        } else if let Ok(_) = reciever.try_recv() {
                            should_run = false;
                        } else {
                            parker.park();
                        }
                    }
                }),
                unparker,
            ));
            info!("After spawned thread");
        }
        return Self {
            connection: send,
            handles,
            new_queue: new_queue.clone(),
            outputs,
            initial_state,
            _phantom_data: PhantomData::default(),
        };
    }

    // Generate a random string to be used as a batchID. Change this to an auto-incremented u32 at
    // at some point.
    fn gen_key() -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz";
        const KEY_LEN: usize = 10;
        let mut rng = StdRand::default();

        let key: String = (0..KEY_LEN)
            .map(|_| {
                let idx = rng.next_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        return key;
    }

    pub fn new_batch(&mut self) -> String {
        let mut key = Self::gen_key();
        while self.outputs.contains_key(key.as_str()) {
            key = Self::gen_key();
        }
        self.outputs.insert(
            key.clone(),
            Batch {
                output: Arc::new(SkipMap::new()),
                size: 0,
            },
        );
        key
    }

    pub fn add_to_batch(&mut self, id: &String, input: &'a [u8], order: usize) {
        let batch = self.outputs.get_mut(id).unwrap();
        batch.size += 1;
        self.new_queue.push(WorkUnit(order, input, (*batch).output.clone()));
        for (_, unparker) in &mut self.handles {
            unparker.unpark();
        }
    }

    fn print_lexer_state_list(list: &Vec<Token>) {
        let mut builder = String::new();
        for x in list {
            builder.push_str(format!("{:?} ", x).as_str());
        }
        trace!("{}", builder);
    }

    pub fn collect_batch(&mut self, id: String) -> LinkedList<Vec<usize>> {
        let batch: Batch = self.outputs.remove(id.as_str()).unwrap();

        // Spin until threads have finished lexing.
        while batch.size != batch.output.len() {}

        // Append first item in list to output
        let mut result: LinkedList<Vec<usize>> = LinkedList::new();

        // For some unknown (probably data-race) reason, if there is only one thread,
        // it will intermittently fail to pop the top of the skiplist even though its
        // .len() function shows that its not empty. Keeping pop'in till were not nothing.
        let mut first = batch.output.pop_front();
        while first.is_none() {
            first = batch.output.pop_front();
        }

        let first = first.unwrap();
        let mut first = first.value().write().unwrap().lists.take().unwrap();
        let start_state_output = first.remove(&self.initial_state).unwrap();
        result.push_back(start_state_output.list);

        let mut previous_finish_state = self.initial_state;
        for x in batch.output.iter() {
            let mut val = x.value().write().unwrap();
            let mut found_match = false;
            for (start_state, partial_output) in val.lists.take().unwrap() {
                trace!("Checking {:?} -> {:?} : ", previous_finish_state, start_state);
                if previous_finish_state == start_state {
                    trace!("yes");

                    found_match = true;
                    previous_finish_state = partial_output.finish_state;
                    result.push_back(partial_output.list);
                    break;
                } else {
                    trace!("no");
                }
            }
            if !found_match {
                panic!("ERROR: finished on {:?}", previous_finish_state);
            }
        }
        return result;
    }

    pub fn kill(mut self) {
        for (_, unparker) in &mut self.handles {
            self.connection.send(true).unwrap();
            unparker.unpark();
        }
        while !self.handles.is_empty() {
            let mut left_overs = Vec::new();
            for (handle, u) in self.handles {
                if handle.is_finished() {
                    handle.join().unwrap();
                } else {
                    u.unpark();
                    left_overs.push((handle, u));
                }
            }
            self.handles = left_overs;
            if !self.handles.is_empty() {
                for i in 0..self.handles.len() {
                    if let Some(t) = self.handles.get_mut(i) {
                        if t.0.is_finished() {
                            self.handles.remove(i);
                        }
                    }
                }
            }
            thread::sleep(Duration::new(0, 1000))
        }
    }
}

// pub fn lex(input: &str, grammar: &OpGrammar, threads: usize) -> Result<LinkedList<Vec<(Token, JsonData)>>, Box<dyn Error>> {
//     let mut tokens: LinkedList<Vec<(Token, JsonData)>> = LinkedList::new();
//     {
//         thread::scope(|s| {
//             let mut lexer: ParallelLexer<JsonLexerState, JsonLexer, JsonData> =
//                 ParallelLexer::new(&grammar, s, threads, &[JsonLexerState::Start, JsonLexerState::InString], JsonLexerState::Start);
//             let batch = lexer.new_batch();
//             lexer.add_to_batch(&batch, input.as_bytes(), 0);
//             tokens = lexer.collect_batch(batch);
//             lexer.kill();
//         });
//     }
//     return Ok(tokens);
// }

// #[test]
// pub fn test_lexer() -> Result<(), Box<dyn Error>> {
//     let grammar = OpGrammar::from("data/grammar/json.g");
//     let t = JsonTokens::new(&grammar.token_reverse);
//
//     let test = |input: &str, expected: Vec<Token>| -> Result<(), Box<dyn Error>> {
//         let mut ll = lex(input, &grammar, 1)?;
//         let mut output = Vec::new();
//         for list in &mut ll {
//             output.append(list);
//         }
//         assert_eq!(output, expected);
//         Ok(())
//     };
//
//     let input = "\
//     {\
//         \"test\": 100\
//     }";
//     let expected = vec![
//         t.lbrace, t.quotes, t.char, t.char, t.char, t.char, t.quotes, t.colon, t.number, t.rbrace,
//     ];
//     test(input, expected)?;
//     Ok(())
// }
