extern crate core;

use std::fs::File;
use std::thread;
use core::grammar::Grammar;
use core::lexer::ParallelLexer;

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

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("lexer_1MB", |b| b.iter(|| bench_lexer("json/1MB.json")));
    c.bench_function("lexer_10MB", |b| b.iter(|| bench_lexer("json/10MB.json")));
    c.bench_function("lexer_100MB", |b| b.iter(|| bench_lexer("json/100MB.json")));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);