use crossbeam::sync::Parker;
use crossbeam::sync::Unparker;
use crossbeam_deque::{Injector, Worker};
use crossbeam_skiplist::SkipMap;
use log::trace;
use memmap::{Mmap, MmapOptions};
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
pub mod fern;
pub mod json;
pub mod lua;

use crate::grammar::{OpGrammar, Token};
use crate::lexer::error::LexerError;
use crate::lexer::json::{JsonData, JsonLexer, JsonLexerState, JsonTokens};
use crate::lexer::lua::{LuaLexer, LuaLexerState};
use crossbeam_queue::SegQueue;

pub struct LexerOutput<T, Data> {
    lists: Option<HashMap<T, LexerPartialOutput<T, Data>>>,
}

#[allow(unused)]
pub struct LexerPartialOutput<T, Data> {
    list: Vec<(Token, Data)>,
    finish_state: T,
    success: bool,
}

pub struct WorkUnit<'a, T, Data>(usize, &'a [u8], Arc<SkipMap<usize, RwLock<LexerOutput<T, Data>>>>);

pub struct ParallelLexer<'a, T, U, Data> {
    handles: Vec<(ScopedJoinHandle<'a, ()>, Unparker)>,
    connection: crossbeam_channel::Sender<bool>,
    new_queue: Arc<SegQueue<WorkUnit<'a, T, Data>>>,
    outputs: HashMap<String, Batch<T, Data>>,
    initial_state: T,
    _phantom_data: PhantomData<U>,
}

pub struct Batch<T, Data> {
    output: Arc<SkipMap<usize, RwLock<LexerOutput<T, Data>>>>,
    size: usize,
}

pub trait LexerInterface<T, Data> {
    fn new(grammar: OpGrammar, start_state: T) -> Self;
    fn consume(&mut self, c: &u8) -> Result<(), LexerError>;
    fn take(self) -> (T, Vec<(Token, Data)>);
}

impl<'a, T, Lexer, Data> ParallelLexer<'a, T, Lexer, Data>
where
    T: Copy + Send + Sync + 'static + Eq + PartialEq + Hash + Debug,
    Data: Send + Sync + 'static + Eq + PartialEq + Hash + Debug,
    Lexer: LexerInterface<T, Data>,
{
    pub fn new(grammar: &OpGrammar, scope: &'a Scope<'a, '_>, threads: usize, possible_start_states: &[T], initial_state: T) -> Self {
        let new_queue: Arc<SegQueue<WorkUnit<T, Data>>> = Arc::new(SegQueue::new());
        let (send, recv) = crossbeam_channel::bounded(threads);
        let outputs: HashMap<String, Batch<T, Data>> = HashMap::new();

        let mut handles = vec![];
        for _ in 0..threads {
            let reciever = recv.clone();
            let new_queue = new_queue.clone();
            let grammar = grammar.clone();
            let start_states: Vec<T> = Vec::from(possible_start_states);
            let parker = Parker::new();
            let unparker = parker.unparker().clone();

            handles.push((
                scope.spawn(move || {
                    let mut should_run = true;
                    while should_run {
                        let task = new_queue.pop();
                        if let Some(task) = task {
                            let mut lexers: Vec<(Lexer, T, bool)> = Vec::new();
                            for state in &start_states {
                                lexers.push((Lexer::new(grammar.clone(), *state), *state, true));
                            }

                            for c in task.1 {
                                for (lexer, _, is_successful) in &mut lexers {
                                    if *is_successful {
                                        if let Err(_) = lexer.consume(c) {
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
                                    if let Err(_) = lexer.consume(&(' ' as u8)) {
                                        *is_successful = false;
                                    }
                                }
                            }

                            let mut map: HashMap<T, LexerPartialOutput<T, Data>> = HashMap::new();
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

    pub fn collect_batch(&mut self, id: String) -> LinkedList<Vec<(Token, Data)>> {
        let batch: Batch<T, Data> = self.outputs.remove(id.as_str()).unwrap();

        // Spin until threads have finished lexing.
        while batch.size != batch.output.len() {}

        // Append first item in list to output
        let mut result: LinkedList<Vec<(Token, Data)>> = LinkedList::new();

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

pub fn split_mmap_into_chunks<'a>(mmap: &'a mut Mmap, step: usize) -> Result<Vec<&'a [u8]>, Box<dyn Error>> {
    let mut indices = vec![];
    let mut i = 0;
    let mut prev = 0;

    while i < mmap.len() {
        if mmap[i] as char != ' ' && mmap[i] as char != '\n' {
            i += 1;
        } else {
            if i + 1 <= mmap.len() {
                i += 1;
            }
            indices.push((prev, i));
            prev = i;
            i += step;
        }
    }
    if prev < mmap.len() {
        indices.push((prev, mmap.len()));
    }

    let mut units = vec![];
    for i in indices {
        units.push(&mmap[i.0..i.1]);
    }
    return Ok(units);
}

pub fn lex(input: &str, grammar: &OpGrammar, threads: usize) -> Result<LinkedList<Vec<(Token, JsonData)>>, Box<dyn Error>> {
    let mut tokens: LinkedList<Vec<(Token, JsonData)>> = LinkedList::new();
    {
        thread::scope(|s| {
            let mut lexer: ParallelLexer<JsonLexerState, JsonLexer, JsonData> =
                ParallelLexer::new(&grammar, s, threads, &[JsonLexerState::Start, JsonLexerState::InString], JsonLexerState::Start);
            let batch = lexer.new_batch();
            lexer.add_to_batch(&batch, input.as_bytes(), 0);
            tokens = lexer.collect_batch(batch);
            lexer.kill();
        });
    }
    return Ok(tokens);
}

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
