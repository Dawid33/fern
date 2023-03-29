use std::collections::{HashMap};
use tinyrand::{StdRand, RandRange};
use std::{fs, iter, thread};
use std::error::Error;
use std::fs::File;
use std::slice::Chunks;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
use std::thread::{JoinHandle, Scope, ScopedJoinHandle, spawn};
use std::time::Instant;
use crossbeam_deque::{Injector, Worker};
use memmap::MmapOptions;

mod lexer;
mod error;

use lexer::*;

struct LexerOutput {}

struct WorkUnit<'a>(BatchId, &'a [u8]);

struct ParallelLexer<'a> {
    handles: Vec<ScopedJoinHandle<'a, ()>>,
    connection: crossbeam_channel::Sender<bool>,
    queue: Arc<Injector<WorkUnit<'a>>>,
    outputs: HashMap<String, Vec<Vec<LexerOutput>>>,
}

struct ParseTree {}

impl ParseTree {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Clone, Debug)]
struct BatchId(String);

impl<'a> ParallelLexer<'a> {
    pub fn new(scope: &'a Scope<'a, '_>, threads: usize) -> Self {
        let queue: Arc<Injector<WorkUnit>> = Arc::new(Injector::new());
        let (send, recv) = crossbeam_channel::bounded(threads);

        let mut handles = vec![];
        for _ in 0..threads {
            let reciever = recv.clone();
            let global = queue.clone();

            handles.push(scope.spawn(move || {
                let worker: Worker<WorkUnit> = Worker::new_fifo();

                let mut token_buf = Vec::new();
                let mut should_run = true;
                while should_run {
                    let task: Option<WorkUnit<'a>> = worker.pop().or_else(|| {
                        iter::repeat_with(|| {
                            global.steal_batch_and_pop(&worker)
                        }).find(|s| !s.is_retry()).and_then(|s| s.success())
                    });
                    if let Some(task) = task {
                        let mut lexer : JsonLexer = JsonLexer::new(&mut token_buf);

                        for c in task.1 {
                            lexer.consume(c).unwrap();
                        }
                        println!("Created {} tokens.", token_buf.len());
                        token_buf.clear();
                    } else if let Ok(_) = reciever.try_recv() {
                        should_run = false;
                    } else {
                        continue;
                    }
                }
            }));
        }
        return Self { connection: send, handles, queue: queue.clone(), outputs: HashMap::new() };
    }

    pub fn start(&mut self) {
    }

    fn gen_key() -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz";
        const KEY_LEN: usize = 10;
        let mut rng = StdRand::default();

        let key: String = (0..KEY_LEN).map(|_| {
            let idx = rng.next_range(0..CHARSET.len());
            CHARSET[idx] as char
        }).collect();
        return key;
    }

    pub fn new_batch(&mut self) -> BatchId {
        let mut key = Self::gen_key();
        while self.outputs.contains_key(key.as_str()) {
            key = Self::gen_key();
        }
        self.outputs.insert(key.clone(), Vec::new());
        BatchId(key)
    }

    pub fn add_to_batch(&mut self, id: &BatchId, input: &'a [u8]) {
        self.queue.push(WorkUnit(id.clone(), input));
    }

    pub fn collect_batch(&mut self, id: BatchId) -> Vec<Vec<LexerOutput>> {
        let x = self.outputs.remove(id.0.as_str());
        return x.unwrap();
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

fn parallel() -> Result<(), Box<dyn Error>> {
    let threads = 12;
    let now = Instant::now();
    let file = File::open("json/1GB.json")?;
    let x: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };

    let mut indices = vec![];
    let step = 1000;
    indices.push((0, step));
    let mut i = 0;
    let mut prev = 0;

    while i < x.len() {
        if x[i] as char != '\n' {
            i += 1;
        } else {
            indices.push((prev, i));
            prev = i;
            i += step;
        }
    }

    let mut units = vec![];
    for i in indices {
        units.push(&x[i.0..i.1]);
    }

    println!("Reading file : {:?}", now.elapsed());
    let now = Instant::now();

    thread::scope(|s| {
        let mut lexer = ParallelLexer::new(s, threads);
        let batch = lexer.new_batch();
        for task in units {
            lexer.add_to_batch(&batch, task);
        }
        lexer.collect_batch(batch);
        lexer.kill();
    });

    println!("Lexing : {:?}", now.elapsed());
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Test lexing in parallel
    parallel().unwrap();

    // let file = File::open("json/1GB.json")?;
    // let mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };
    //
    // let now = Instant::now();
    // thread::scope(|s| {
    //     let mut lexer = ParallelLexer::new(s, 1);
    //     let batch = lexer.new_batch();
    //     lexer.add_to_batch(&batch, &mmap[..]);
    //     lexer.collect_batch(batch);
    //     lexer.kill();
    // });
    // println!("Lexing : {:?}", now.elapsed());

    Ok(())
}
