extern crate core;
extern crate libfern;

use libfern::{
    fern::FernLexer,
    grammar::{
        lg::{LexicalGrammar, StateGraph},
        opg::OpGrammar,
    },
    lexer::{split_mmap_into_chunks, ParallelLexer},
};
use std::error::Error;
use std::fs::File;
use std::thread;
use std::time::Instant;
use std::{collections::LinkedList, io::Read};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use memmap::MmapOptions;

fn bench_parallel_lexing(path: &str, threads: usize) {
    let mut file = File::open("data/grammar/json.lg").unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    let g = LexicalGrammar::from(buf.clone());
    let nfa = StateGraph::from(g.clone());
    let dfa = nfa.convert_to_dfa();
    let table = dfa.build_table();

    let file = File::open(path).unwrap();
    let mut memmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file).unwrap() };
    let chunks = split_mmap_into_chunks(&mut memmap, 6000).unwrap();

    let _ = thread::scope(|s| {
        let mut lexer: ParallelLexer<FernLexer> = ParallelLexer::new(table.clone(), s, threads, &[0], 0);
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
    c.bench_function("json_lexer_1_thread_10MB", |b| b.iter(|| bench_parallel_lexing("data/json/10MB.json", 1)));
    c.bench_function("json_lexer_2_thread_10MB", |b| b.iter(|| bench_parallel_lexing("data/json/10MB.json", 2)));
    c.bench_function("json_lexer_4_thread_10MB", |b| b.iter(|| bench_parallel_lexing("data/json/10MB.json", 4)));
    c.bench_function("json_lexer_8_thread_10MB", |b| b.iter(|| bench_parallel_lexing("data/json/10MB.json", 8)));
    c.bench_function("json_lexer_16_thread_10MB", |b| b.iter(|| bench_parallel_lexing("data/json/10MB.json", 16)));

    // c.bench_function("json_fair_sequential_lexing_10KB", |b| b.iter(|| fair_sequential_lexing("data/json/10KB.json")));
    // c.bench_function("json_fair_sequential_lexing_100KB", |b| b.iter(|| fair_sequential_lexing("data/json/100KB.json")));
    // c.bench_function("json_fair_sequential_lexing_1MB", |b| b.iter(|| fair_sequential_lexing("data/json/1MB.json")));
    // c.bench_function("json_fair_sequential_lexing_10MB", |b| b.iter(|| fair_sequential_lexing("data/json/10MB.json")));
    // c.bench_function("json_fair_sequential_lexing_50MB", |b| b.iter(|| fair_sequential_lexing("data/json/50MB.json")));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
