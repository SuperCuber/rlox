use std::{
    env::args,
    io::{stdin, BufRead, Write},
};

use anyhow::{Context, Result};

mod error;
mod expression;
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
    let stdin = stdin();
    let stdin = stdin.lock();

    print!("> ");
    std::io::stdout().flush().unwrap();
    for line in stdin.lines() {
        if let Ok(line) = line {
            match run(line) {
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
    run(std::fs::read_to_string(filename).context("read file")?)?;
    Ok(())
}

fn run(source: String) -> Result<()> {
    let mut scanner = scanner::Scanner::new(source);
    let (tokens, scan_errors) = scanner.tokens();

    if !scan_errors.is_empty() {
        dbg!(scan_errors);
        return Ok(());
    }
    let mut parser = parser::Parser::new(tokens);
    let ast = parser.parse()?;
    interpreter::interpret(ast)?;

    Ok(())
}
