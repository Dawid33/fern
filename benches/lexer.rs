extern crate core;

use std::fs::File;
use std::thread;
use core::grammar::Grammar;
use core::lexer::ParallelLexer;
use std::error::Error;
use std::time::Instant;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use memmap::MmapOptions;

fn bench_lexer_sequential(path: &str) -> Result<(), Box<dyn Error>>{
    let grammar = Grammar::from("json.g");
    let mut tokens: Vec<u8> = Vec::new();
    {
        let file = File::open("test.json")?;
        let mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };
        thread::scope(|s| {
            let mut lexer = ParallelLexer::new(grammar.clone(), s, 1);
            let batch = lexer.new_batch();
            lexer.add_to_batch(&batch, &mmap[..], 0);
            tokens = lexer.collect_batch(batch);
            lexer.kill();
        });
    }
    Ok(())
}

fn bench_parallel_lexing(path: &str, threads: usize) {
    let grammar = Grammar::from("json.g");
    let file = File::open(path).unwrap();
    let mut memmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
    let chunks = core::lexer::split_mmap_into_chunks(&mut memmap, 1000).unwrap();

    thread::scope(|s| {
        let mut lexer = ParallelLexer::new(grammar, s, threads);
        let batch = lexer.new_batch();
        for task in chunks.iter().enumerate() {
            lexer.add_to_batch(&batch, task.1, task.0);
        }
        let output = lexer.collect_batch(batch);
        lexer.kill();
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("lexer_sequential_full", |b| b.iter(|| bench_lexer_sequential("data/full.json")));
    // c.bench_function("lexer_sequential_10MB", |b| b.iter(|| bench_lexer_sequential("json/10MB.json")));
    // c.bench_function("parallel_lexer_full", |b| b.iter(|| bench_parallel_lexing("data/full.json", 12)));
    // c.bench_function("parallel_lexer_10MB", |b| b.iter(|| bench_parallel_lexing("json/10MB.json", 12)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);