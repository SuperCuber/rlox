use std::io::{stdin, BufRead, Write};

use crate::vm::VM;

mod chunk;
mod compiler;
mod scanner;
mod value;
mod vm;

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [] => repl(),
        [f] => run_file(f),
        _ => todo!(),
    }
}

fn repl() {
    let stdin = stdin();
    let stdin = stdin.lock();
    print!("> ");
    std::io::stdout().flush().unwrap();
    for line in stdin.lines() {
        if let Ok(line) = line {
            VM::new(line).unwrap().run().unwrap();
            print!("> ");
            std::io::stdout().flush().unwrap();
        } else {
            println!("bye");
            break;
        }
    }
}

fn run_file(filename: &str) {
    let source = std::fs::read_to_string(filename).unwrap();
    VM::new(source).unwrap().run().unwrap();
}
