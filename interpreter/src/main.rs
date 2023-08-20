use std::{
    env::args,
    io::{stdin, BufRead, Write},
    iter::repeat,
};

use anyhow::{Context, Result};
use error::LoxError;
use interpreter::Interpreter;

mod ast;
mod environment;
mod error;
mod interpreter;
mod parser;
mod resolver;
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
            if let Err(errs) = run(line, &mut interpreter, true) {
                for err in errs {
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
    let source = std::fs::read_to_string(filename).context("read source file")?;
    if let Err(errs) = run(source.clone(), &mut interpreter, false) {
        for err in errs {
            eprintln!("{}", err);
            if let Some((line, col)) = err.location() {
                let line_text = source
                    .split('\n')
                    .nth(line - 1)
                    .expect("find error line in source code");
                eprintln!("{line_text}");
                let padding = repeat(' ').take(col - 1).collect::<String>();
                eprintln!("{padding}^");
            }
        }
    }
    Ok(())
}

fn run(
    source: String,
    interpreter: &mut Interpreter,
    allow_single_expression: bool,
) -> Result<(), Vec<LoxError>> {
    let mut scanner = scanner::Scanner::new(source);
    let tokens = scanner
        .tokens()
        .map_err(|e| e.into_iter().map(Into::into).collect::<Vec<LoxError>>())?;

    if allow_single_expression {
        // Try to parse as a single expression
        let parser = parser::Parser::new(tokens.clone());
        let expr = parser
            .parse_expression()
            .map_err(|e| e.into_iter().map(Into::into).collect::<Vec<LoxError>>());
        if let Ok(expr) = expr {
            let mut resolver = resolver::Resolver::new();
            let expr = resolver
                .resolve_expr(expr)
                .map_err(|e| e.into_iter().map(Into::into).collect::<Vec<LoxError>>())?;
            let value = interpreter.evaluate(expr).map_err(|e| vec![e.into()])?;
            println!("{}", value);
            return Ok(());
        }
        // Try to parse as a program now
    }

    let parser = parser::Parser::new(tokens);
    let ast = parser
        .parse()
        .map_err(|e| e.into_iter().map(Into::into).collect::<Vec<LoxError>>())?;
    let mut resolver = resolver::Resolver::new();
    let ast = resolver
        .resolve(ast)
        .map_err(|e| e.into_iter().map(Into::into).collect::<Vec<LoxError>>())?;

    interpreter.interpret(ast).map_err(|e| vec![e.into()])?;
    Ok(())
}
