use std::{
    env::args,
    io::{stdin, BufRead, Write},
};

use anyhow::{Context, Result};
use interpreter::Interpreter;

mod ast;
mod environment;
mod error;
mod interpreter;
mod parser;
mod scanner;
mod token;
mod value;

fn main() -> Result<()> {
    anyhow::ensure!(args().len() <= 2, "Too many arguments given");

    match args().nth(1) {
        Some(filename) => run_file(filename),
        None => run_prompt(),
    }?;

    Ok(())
}

fn run_prompt() -> Result<()> {
    let mut interpreter = interpreter::Interpreter::new();

    let stdin = stdin();
    let stdin = stdin.lock();
    print!("> ");
    std::io::stdout().flush().unwrap();
    for line in stdin.lines() {
        if let Ok(line) = line {
            match run(line, &mut interpreter) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("{}", err);
                }
            };
            print!("> ");
            std::io::stdout().flush().unwrap();
        } else {
            println!("End of input. Goodbye!");
            break;
        }
    }
    Ok(())
}

fn run_file(filename: String) -> Result<()> {
    let mut interpreter = interpreter::Interpreter::new();
    run(
        std::fs::read_to_string(filename).context("read file")?,
        &mut interpreter,
    )?;
    Ok(())
}

fn run(source: String, interpreter: &mut Interpreter) -> Result<()> {
    let mut scanner = scanner::Scanner::new(source);
    let (tokens, scan_errors) = scanner.tokens();

    if !scan_errors.is_empty() {
        for err in scan_errors {
            eprintln!("{}", err);
        }
        return Ok(());
    }
    let parser = parser::Parser::new(tokens);
    match parser.parse() {
        Ok(ast) => interpreter.interpret(ast)?,
        Err(errs) => {
            for err in errs {
                eprintln!("{}", err);
            }
        }
    };

    Ok(())
}
