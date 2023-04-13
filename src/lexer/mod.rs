use std::collections::{HashMap, LinkedList};
use std::io::{stdout, Write};
use crossbeam_deque::{Injector, Worker};
use std::{iter, thread};
use std::error::Error;
use std::fs::File;
use std::ops::Deref;
use std::sync::{Arc, RwLock};
use std::task::ready;
use std::thread::{Scope, ScopedJoinHandle};
use std::time::{Duration, Instant};
use crossbeam_skiplist::SkipMap;
use log::trace;
use memmap::{Mmap, MmapOptions};
use tinyrand::{RandRange, StdRand};
pub mod fern;

use fern::FernLexerState::InString;
use crate::grammar::Grammar;
use crate::lexer::fern::{FernLexer, FernTokens, FernLexerState};
use crate::lexer::fern::FernLexerState::InName;
use crate::lexer::json::{JsonLexer, JsonLexerState, JsonTokens};

pub mod error;
pub mod json;

pub struct LexerOutput {
    lists: Option<HashMap<JsonLexerState, LexerPartialOutput>>,
}

#[allow(unused)]
pub struct LexerPartialOutput {
    list: Vec<u8>,
    finish_state: JsonLexerState,
    success: bool,
}

pub struct WorkUnit<'a>(usize, &'a [u8], Arc<SkipMap<usize, RwLock<LexerOutput>>>);

pub struct ParallelLexer<'a> {
    handles: Vec<ScopedJoinHandle<'a, ()>>,
    connection: crossbeam_channel::Sender<bool>,
    queue: Arc<Injector<WorkUnit<'a>>>,
    outputs: HashMap<String, Batch>,
}

pub struct Batch {
    output: Arc<SkipMap<usize, RwLock<LexerOutput>>>,
    size: usize,
}

impl<'a> ParallelLexer<'a> {
    pub fn new(grammar: Grammar, scope: &'a Scope<'a, '_>, threads: usize) -> Self {
        let queue: Arc<Injector<WorkUnit>> = Arc::new(Injector::new());
        let (send, recv) = crossbeam_channel::bounded(threads);
        let outputs : HashMap<String, Batch> = HashMap::new();

        let mut handles = vec![];
        for _ in 0..threads {
            let reciever = recv.clone();
            let global = queue.clone();
            let grammar = grammar.clone();

            handles.push(scope.spawn(move || {
                let worker: Worker<WorkUnit> = Worker::new_fifo();

                let mut should_run = true;
                while should_run {
                    let task: Option<WorkUnit<'a>> = worker.pop().or_else(|| {
                        iter::repeat_with(|| global.steal_batch_and_pop(&worker))
                            .find(|s| !s.is_retry())
                            .and_then(|s| s.success())
                    });
                    if let Some(task) = task {
                        let mut token_buf = Vec::new();
                        let mut token_buf_string = Vec::new();
                        let mut lexer_start: JsonLexer = JsonLexer::new(grammar.clone(), &mut token_buf, JsonLexerState::Start);
                        let mut lexer_string: JsonLexer = JsonLexer::new(grammar.clone(), &mut token_buf_string, JsonLexerState::InString);
                        let mut start = true;
                        let mut string = true;

                        for c in task.1 {
                            if start {
                                if let Err(_) = lexer_start.consume(c) { start = false; }
                            }
                            if string {
                                if let Err(_) = lexer_string.consume(c) { string = false; }
                            }
                        }

                        let mut map: HashMap<JsonLexerState, LexerPartialOutput> = HashMap::new();
                        map.insert(JsonLexerState::Start, LexerPartialOutput {success: start, finish_state: lexer_start.state, list: token_buf});
                        map.insert(JsonLexerState::InString, LexerPartialOutput {success: string, finish_state: lexer_string.state, list: token_buf_string});

                        task.2.insert(task.0, RwLock::new(LexerOutput { lists: Some(map)}));
                    } else if let Ok(_) = reciever.try_recv() {
                        should_run = false;
                    } else {
                        continue;
                    }
                }
            }));
        }
        return Self {
            connection: send,
            handles,
            queue: queue.clone(),
            outputs,
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
        self.outputs.insert(key.clone(), Batch {
            output: Arc::new(SkipMap::new()),
            size: 0,
        });
        key
    }

    pub fn add_to_batch(&mut self, id: &String, input: &'a [u8], order: usize) {
        let mut batch= self.outputs.get_mut(id).unwrap();
        batch.size += 1;
        self.queue.push(WorkUnit(order, input, (*batch).output.clone()));
    }

    // Fix this mess
    pub fn collect_batch(&mut self, id: String) -> LinkedList<Vec<u8>> {
        let x : Batch = self.outputs.remove(id.as_str()).unwrap();

        // Spin until threads have finished lexing.
        while x.size != x.output.len() { }

        // Append first item in list to output
        let mut result: LinkedList<Vec<u8>> = LinkedList::new();
        let mut first = x.output.pop_front();


        while first.is_none() {
            first = x.output.pop_front();
        }
        let mut first = first.unwrap();
        let mut first = first.value().write().unwrap().lists.take().unwrap();
        let mut start_state_output = first.remove(&JsonLexerState::Start).unwrap();
        result.push_back(start_state_output.list);

        // for x in &start_state_output.list {
        //     trace!("{:?} ", x);
        // }
        // trace!("");

        let mut previous_finish_state = JsonLexerState::Start;
        for x in x.output.iter() {
            let mut val = x.value().write().unwrap();
            let mut found_match = false;
            for (start_state, partial_output) in val.lists.take().unwrap() {
                trace!("Checking {:?} -> {:?} : ", previous_finish_state, start_state);
                if previous_finish_state == start_state {
                    trace!("yes");

                    for x in &partial_output.list {
                        trace!("{:?} ", x);
                    }
                    trace!("\n");
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

    pub fn kill(self) {
        for _ in 0..self.handles.len() {
            self.connection.send(true).unwrap();
        }
        for x in self.handles {
            let _ = x.join();
        }
    }
}

pub fn split_mmap_into_chunks<'a>(mmap: &'a mut Mmap, step: usize) -> Result<Vec<&'a [u8]>, Box<dyn Error>>{
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

pub fn lex(input: &str, grammar: &Grammar, threads: usize) -> Result<LinkedList<Vec<u8>>, Box<dyn Error>> {
    let mut tokens: LinkedList<Vec<u8>> = LinkedList::new();
    {
        thread::scope(|s| {
            let mut lexer = ParallelLexer::new(grammar.clone(), s, threads);
            let batch = lexer.new_batch();
            lexer.add_to_batch(&batch, input.as_bytes(), 0);
            tokens = lexer.collect_batch(batch);
            lexer.kill();
        });
    }
    return Ok(tokens);
}

// #[test]
// pub fn test_lexer() -> Result<(), Box<dyn Error>>{
//     let grammar = Grammar::from("json.g");
//     let t = JsonTokens::new(&grammar.tokens_reverse);
//
//     let test = |input: &str, expected: Vec<u8>| -> Result<(), Box<dyn Error>>{
//         let output = lex(input, &grammar,1)?;
//         assert_eq!(output, expected);
//         Ok(())
//     };
//
//     let input = "\
//     {\
//         \"test\": 100\
//     }";
//     let expected = vec![t.lbrace, t.quotes, t.char, t.char, t.char, t.char, t.quotes, t.colon, t.number, t.rbrace];
//     test(input, expected)?;
//     Ok(())
// }
