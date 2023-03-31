extern crate core;

use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::Instant;

use memmap::MmapOptions;
use crate::lexer::ParallelLexer;

mod error;
mod grammar;
mod json;
mod lexer;
mod parser;

fn parallel() -> Result<(), Box<dyn Error>> {
    let threads = 2;
    let now = Instant::now();
    let file = File::open("json/1KB.json")?;
    let x: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };

    let mut indices = vec![];
    let step = 100;
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
        for task in units.iter().enumerate() {
            lexer.add_to_batch(&batch, task.1, task.0);
        }
        lexer.collect_batch(batch);
        lexer.kill();
    });

    println!("Lexing : {:?}", now.elapsed());
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Test lexing in parallel
    if true {
        // let _ = grammar::Grammar::json_grammar();
        parallel().unwrap();
    }

    // let file = File::open("test.json")?;
    // let mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };
    //
    // let now = Instant::now();
    // thread::scope(|s| {
    //     let mut lexer = ParallelLexer::new(s, 1);
    //     let batch = lexer.new_batch();
    //     lexer.add_to_batch(&batch, &mmap[..], 0);
    //     lexer.collect_batch(batch);
    //     lexer.kill();
    // });
    // println!("Total Lexing Time: {:?}", now.elapsed());

    Ok(())
}
