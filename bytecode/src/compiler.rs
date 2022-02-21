use crate::{
    chunk::Chunk,
    scanner::{CodeToken, Scanner, Token},
};

pub fn compile(source: String) -> Result<Chunk, ()> {
    let mut scanner = Scanner::new(source);
    let mut line = -1isize;
    loop {
        match scanner.scan_token() {
            Ok(CodeToken {
                token: Token::Eof, ..
            }) => break,
            Ok(token) => {
                if token.line as isize != line {
                    print!("{:4} ", token.line);
                    line = token.line as isize;
                } else {
                    print!("   | ");
                }
                println!("{:?}, '{}'", token.token, token.lexeme);
            }
            Err(e) => {
                println!("{:4} Error: {}", e.line, e.error);
                break;
            }
        }
    }
    todo!()
}

struct Compiler {}

impl Compiler {
    fn new() -> Self {
        Self {}
    }
}
