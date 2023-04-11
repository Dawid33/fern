extern crate core;

use std::fs::File;
use std::thread;
use core::grammar::Grammar;
use core::lexer::ParallelLexer;
use std::error::Error;
use std::time::Instant;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use memmap::MmapOptions;

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
    c.bench_function("lexer_1_thread_1MB", |b| b.iter(|| bench_parallel_lexing("json/1MB.json", 1)));
    c.bench_function("lexer_2_thread_1MB", |b| b.iter(|| bench_parallel_lexing("json/1MB.json", 2)));
    c.bench_function("lexer_4_thread_1MB", |b| b.iter(|| bench_parallel_lexing("json/1MB.json", 4)));
    c.bench_function("lexer_8_thread_1MB", |b| b.iter(|| bench_parallel_lexing("json/1MB.json", 8)));
    c.bench_function("lexer_16_thread_1MB", |b| b.iter(|| bench_parallel_lexing("json/1MB.json", 16)));

    c.bench_function("lexer_1_thread_10MB", |b| b.iter(|| bench_parallel_lexing("json/10MB.json", 1)));
    c.bench_function("lexer_2_thread_10MB", |b| b.iter(|| bench_parallel_lexing("json/10MB.json", 2)));
    c.bench_function("lexer_4_thread_10MB", |b| b.iter(|| bench_parallel_lexing("json/10MB.json", 4)));
    c.bench_function("lexer_8_thread_10MB", |b| b.iter(|| bench_parallel_lexing("json/10MB.json", 8)));
    c.bench_function("lexer_16_thread_10MB", |b| b.iter(|| bench_parallel_lexing("json/10MB.json", 16)));

    c.bench_function("lexer_1_thread_100MB", |b| b.iter(|| bench_parallel_lexing("json/100MB.json", 1)));
    c.bench_function("lexer_2_thread_100MB", |b| b.iter(|| bench_parallel_lexing("json/100MB.json", 2)));
    c.bench_function("lexer_4_thread_100MB", |b| b.iter(|| bench_parallel_lexing("json/100MB.json", 4)));
    c.bench_function("lexer_8_thread_100MB", |b| b.iter(|| bench_parallel_lexing("json/100MB.json", 8)));
    c.bench_function("lexer_16_thread_100MB", |b| b.iter(|| bench_parallel_lexing("json/100MB.json", 16)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);