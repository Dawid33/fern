use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::thread;
use std::time::Instant;

use memmap::MmapOptions;

#[test]
fn parallel_lexing() -> Result<(), Box<dyn Error>> {
    let threads = 12;
    let now = Instant::now();
    let file = File::open("json/10KB.json")?;
    let x: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };

    let mut indices = vec![];
    let step = 10;
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

    println!("Reading file : {:?}", now.elapsed());
    let mut now = Instant::now();

    thread::scope(|s| {
        let mut lexer = ParallelLexer::new(s, threads);
        let batch = lexer.new_batch();
        for task in units.iter().enumerate() {
            lexer.add_to_batch(&batch, task.1, task.0);
        }
        let output = lexer.collect_batch(batch, &mut now);
        lexer.kill();

        let mut lexer = ParallelLexer::new(s, 1);
        let batch = lexer.new_batch();
        for task in units.iter().enumerate() {
            lexer.add_to_batch(&batch, task.1, task.0);
        }
        let sequential = lexer.collect_batch(batch, &mut now);
        lexer.kill();

        assert_eq!(output, sequential);
    });
    Ok(())
}
