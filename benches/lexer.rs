extern crate core;

use core::grammar::Grammar;
use core::lexer::json::*;
use core::lexer::*;
use std::collections::LinkedList;
use std::error::Error;
use std::fs::File;
use std::thread;
use std::time::Instant;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use memmap::MmapOptions;

fn fair_sequential_lexing(path: &str) -> Result<(), Box<dyn Error>> {
    let grammar = Grammar::from("data/grammar/json.g");
    let mut tokens: LinkedList<Vec<u8>> = LinkedList::new();
    {
        let file = File::open(path)?;
        let mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };
        thread::scope(|s| {
            let mut lexer: ParallelLexer<JsonLexerState, JsonLexer> = ParallelLexer::new(
                grammar.clone(),
                s,
                1,
                &[JsonLexerState::Start, JsonLexerState::InString],
                JsonLexerState::Start,
            );
            let batch = lexer.new_batch();
            lexer.add_to_batch(&batch, &mmap[..], 0);
            tokens = lexer.collect_batch(batch);
            lexer.kill();
        });
    }
    Ok(())
}

fn bench_parallel_lexing(path: &str, threads: usize) {
    let grammar = Grammar::from("data/grammar/json.g");
    let file = File::open(path).unwrap();
    let mut memmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
    let chunks = core::lexer::split_mmap_into_chunks(&mut memmap, 6000).unwrap();

    let _ = thread::scope(|s| {
        let mut lexer: ParallelLexer<JsonLexerState, JsonLexer> = ParallelLexer::new(
            grammar.clone(),
            s,
            threads,
            &[JsonLexerState::Start, JsonLexerState::InString],
            JsonLexerState::Start,
        );
        let batch = lexer.new_batch();
        for task in chunks.iter().enumerate() {
            lexer.add_to_batch(&batch, task.1, task.0);
        }
        let tokens = lexer.collect_batch(batch);
        lexer.kill();
        tokens
    });
}

#[rustfmt::skip]
fn criterion_benchmark(c: &mut Criterion) {
    // c.bench_function("lexer_1_thread_1MB", |b| b.iter(|| bench_parallel_lexing("data/json/1MB.json", 1)));
    // c.bench_function("lexer_2_thread_1MB", |b| b.iter(|| bench_parallel_lexing("data/json/1MB.json", 2)));
    // c.bench_function("lexer_4_thread_1MB", |b| b.iter(|| bench_parallel_lexing("data/json/1MB.json", 4)));
    // c.bench_function("lexer_8_thread_1MB", |b| b.iter(|| bench_parallel_lexing("data/json/1MB.json", 8)));
    // c.bench_function("lexer_16_thread_1MB", |b| b.iter(|| bench_parallel_lexing("data/json/1MB.json", 16)));
    //
    // c.bench_function("lexer_1_thread_10MB", |b| b.iter(|| bench_parallel_lexing("data/json/10MB.json", 1)));
    // c.bench_function("lexer_2_thread_10MB", |b| b.iter(|| bench_parallel_lexing("data/json/10MB.json", 2)));
    // c.bench_function("lexer_4_thread_10MB", |b| b.iter(|| bench_parallel_lexing("data/json/10MB.json", 4)));
    // c.bench_function("lexer_8_thread_10MB", |b| b.iter(|| bench_parallel_lexing("data/json/10MB.json", 8)));
    // c.bench_function("lexer_16_thread_10MB", |b| b.iter(|| bench_parallel_lexing("data/json/10MB.json", 16)));
    //
    // c.bench_function("lexer_1_thread_100MB", |b| b.iter(|| bench_parallel_lexing("data/json/100MB.json", 1)));
    // c.bench_function("lexer_2_thread_100MB", |b| b.iter(|| bench_parallel_lexing("data/json/100MB.json", 2)));
    // c.bench_function("lexer_4_thread_100MB", |b| b.iter(|| bench_parallel_lexing("data/json/100MB.json", 4)));
    // c.bench_function("lexer_8_thread_100MB", |b| b.iter(|| bench_parallel_lexing("data/json/100MB.json", 8)));
    // c.bench_function("lexer_16_thread_100MB", |b| { b.iter(|| bench_parallel_lexing("data/json/100MB.json", 16)) });
    //
    // c.bench_function("fair_sequential_lexing", |b| b.iter(|| fair_sequential_lexing("data/json/1MB.json")));
    // c.bench_function("fair_sequential_lexing", |b| b.iter(|| fair_sequential_lexing("data/json/10MB.json")));
    // c.bench_function("fair_sequential_lexing", |b| b.iter(|| fair_sequential_lexing("data/json/100MB.json")));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
