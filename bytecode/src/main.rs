use std::io::{stdin, BufRead, Write};

use chunk::{Chunk, OpCode};

use crate::vm::VM;

mod chunk;
mod compiler;
mod scanner;
mod value;
mod vm;

fn main() {
    // let mut chunk = Chunk::new();
    // let const1 = chunk.add_constant(3.0);
    // let const2 = chunk.add_constant(5.0);
    // chunk.write_code(OpCode::Constant(const1), 2);
    // chunk.write_code(OpCode::Constant(const2), 2);
    // chunk.write_code(OpCode::Add, 3);
    // chunk.write_code(OpCode::Return, 5);
    // chunk.disassemble("testerino");
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
