use std::collections::HashMap;
use std::io::{stdout, Write};
use crate::json::{JsonLexer, JsonToken, LexerState};
use crossbeam_deque::{Injector, Worker};
use std::iter;
use std::sync::Arc;
use std::thread::{Scope, ScopedJoinHandle};
use std::time::{Duration, Instant};
use crossbeam_skiplist::SkipMap;
use tinyrand::{RandRange, StdRand};
use crate::json::JsonToken::{Start};
use crate::json::LexerState::InString;

pub struct LexerOutput {
    lists: HashMap<LexerState, LexerPartialOutput>,
}

#[allow(unused)]
pub struct LexerPartialOutput {
    list: Vec<JsonToken>,
    finish_state: LexerState,
    success: bool,
}

pub struct WorkUnit<'a>(usize, &'a [u8], Arc<SkipMap<usize, LexerOutput>>);

pub struct ParallelLexer<'a> {
    handles: Vec<ScopedJoinHandle<'a, ()>>,
    connection: crossbeam_channel::Sender<bool>,
    queue: Arc<Injector<WorkUnit<'a>>>,
    outputs: HashMap<String, Batch>,
}

pub struct Batch {
    output: Arc<SkipMap<usize, LexerOutput>>,
    size: usize,
}

impl<'a> ParallelLexer<'a> {
    pub fn new(scope: &'a Scope<'a, '_>, threads: usize) -> Self {
        let queue: Arc<Injector<WorkUnit>> = Arc::new(Injector::new());
        let (send, recv) = crossbeam_channel::bounded(threads);
        let outputs : HashMap<String, Batch> = HashMap::new();

        let mut handles = vec![];
        for _ in 0..threads {
            let reciever = recv.clone();
            let global = queue.clone();

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
                        let mut lexer_start: JsonLexer = JsonLexer::new(&mut token_buf, LexerState::Start);
                        let mut lexer_string: JsonLexer = JsonLexer::new(&mut token_buf_string, InString);
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

                        let mut map: HashMap<LexerState, LexerPartialOutput> = HashMap::new();
                        map.insert(LexerState::Start, LexerPartialOutput {success: start, finish_state: lexer_start.state, list: token_buf});
                        map.insert(InString, LexerPartialOutput {success: string, finish_state: lexer_string.state, list: token_buf_string});

                        task.2.insert(task.0, LexerOutput { lists: map});
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

    #[allow(unused)]
    pub fn start(&mut self) {}

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
    pub fn collect_batch(&mut self, id: String, time: &mut Instant) -> Vec<JsonToken> {
        let x : Batch = self.outputs.remove(id.as_str()).unwrap();

        // Spin until threads have finished lexing.
        while x.size != x.output.len() { }
        println!("Workers lexing: {:?}", time.elapsed());
        *time = Instant::now();

        // Append first item in list to output
        let mut result: Vec<JsonToken> = Vec::new();
        // let first = x.output.pop_front().unwrap();
        // let first = first.value();
        // let mut start_state_output = &first.lists.get(&LexerState::Start).unwrap();
        // result.append(&mut start_state_output.list.clone());
        //
        // for x in &start_state_output.list {
        //     print!("{:?} ", x);
        // }
        // println!();

        let mut previous_finish_state = LexerState::Start;
        for x in x.output.iter() {
            let val: &LexerOutput= x.value();
            let mut found_match = false;
            for (start_state, partial_output) in &val.lists {
                // print!("Checking {:?} -> {:?} : ", previous_finish_state, *start_state);
                if previous_finish_state == *start_state {
                    // println!("yes");

                    // for x in &partial_output.list {
                        // print!("{:?} ", x);
                    // }
                    // println!("\n");
                    found_match = true;
                    previous_finish_state = partial_output.finish_state;
                    result.append(&mut partial_output.list.clone());
                    break;
                } else {
                    // println!("no");
                }
            }
            if !found_match {
                panic!("ERROR: finished on {:?}", previous_finish_state);
            }
        }


        println!("Joining up work: {:?}", time.elapsed());
        *time = Instant::now();
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
