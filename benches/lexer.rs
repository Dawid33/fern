extern crate core;

use std::fs::File;
use std::thread;
use core::grammar::Grammar;
use core::lexer::ParallelLexer;
use std::error::Error;
use std::time::Instant;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use memmap::MmapOptions;

fn bench_lexer(path: &str) {
    let grammar = Grammar::from("json.g");
    let mut tokens: Vec<u8> = Vec::new();
    {
        let file = File::open(path).unwrap();
        let mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
        thread::scope(|s| {
            let mut lexer = ParallelLexer::new(grammar.clone(), s, 1);
            let batch = lexer.new_batch();
            lexer.add_to_batch(&batch, &mmap[..], 0);
            tokens = lexer.collect_batch(batch);
            lexer.kill();
        });
    }
}

fn bench_parallel_lexing(path: &str) {
    let grammar = Grammar::from("json.g");
    let threads = 12;
    let file = File::open(path).unwrap();
    let x: memmap::Mmap = unsafe { MmapOptions::new().map(&file).unwrap() };

    let mut indices = vec![];
    let step = 1000;
    let mut i = 0;
    let mut prev = 0;

    while i < x.len() {
        if x[i] as char != ' ' && x[i] as char != '\n' {
            i += 1;
        } else {
            if i + 1 <= x.len() {
                i += 1;
            }
            indices.push((prev, i));
            prev = i;
            i += step;
        }
    }
    if prev < x.len() {
        indices.push((prev, x.len()));
    }

    let mut units = vec![];
    for i in indices {
        units.push(&x[i.0..i.1]);
    }

    thread::scope(|s| {
        let mut lexer = ParallelLexer::new(grammar, s, threads);
        let batch = lexer.new_batch();
        for task in units.iter().enumerate() {
            lexer.add_to_batch(&batch, task.1, task.0);
        }
        let output = lexer.collect_batch(batch);
        lexer.kill();
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("lexer_1MB", |b| b.iter(|| bench_lexer("json/1MB.json")));
    c.bench_function("lexer_10MB", |b| b.iter(|| bench_lexer("json/10MB.json")));
    c.bench_function("parallel_lexer_1MB", |b| b.iter(|| bench_parallel_lexing("json/1MB.json")));
    c.bench_function("parallel_lexer_10MB", |b| b.iter(|| bench_parallel_lexing("json/10MB.json")));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);