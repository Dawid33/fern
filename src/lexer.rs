use std::collections::HashMap;
use std::io::{stdout, Write};
use crate::json::{JsonLexer, JsonToken, LexerState};
use crossbeam_deque::{Injector, Worker};
use std::iter;
use std::sync::Arc;
use std::thread::{Scope, ScopedJoinHandle};
use crossbeam_skiplist::SkipMap;
use tinyrand::{RandRange, StdRand};
use crate::json::LexerState::InString;

pub struct LexerOutput {
    lists: Vec<LexerPartialOutput>,
}

#[allow(unused)]
pub struct LexerPartialOutput {
    list: Vec<JsonToken>,
    start_state: LexerState,
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

                        task.2.insert(task.0, LexerOutput {
                            lists: vec! [
                                LexerPartialOutput {list: token_buf, start_state: LexerState::Start, success: start},
                                LexerPartialOutput {list: token_buf_string, start_state: InString, success: string}
                            ]
                        });
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

    pub fn collect_batch(&mut self, id: String) -> Vec<LexerOutput> {
        let x : Batch = self.outputs.remove(id.as_str()).unwrap();

        // Spin until threads have finished lexing.
        while x.size != x.output.len() { }

        for x in x.output.iter() {
            let val: &LexerOutput= x.value();
            for list in &val.lists {
                println!("{:?} -> {}", list.start_state, list.success);
            }
            println!();
        }

        return Vec::new();
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
