#![allow(dead_code)]

#[derive(Debug)]
pub struct CodeToken {
    pub token: Token,
    pub line: usize,
    pub lexeme: String,
}

#[derive(Debug)]
pub enum Token {
    Literal(Literal),
    Symbol(Symbol),
    Word(Word),
    Eof,
}

#[derive(Debug)]
pub enum Word {
    Identifier(String),
    Keyword(Keyword),
}

#[derive(Debug)]
pub enum Literal {
    String(String),
    Number(f32),
    Boolean,
    Nil,
}

#[derive(Debug)]
pub enum Symbol {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
}

#[derive(Debug)]
pub enum Keyword {
    And,
    Class,
    Else,
    Fun,
    For,
    If,
    Or,
    Print,
    Return,
    Super,
    This,
    Var,
    While,
}

impl Keyword {
    pub fn from_word(word: &str) -> Option<Keyword> {
        Some(match word {
            "and" => Keyword::And,
            "class" => Keyword::Class,
            "else" => Keyword::Else,
            "fun" => Keyword::Fun,
            "for" => Keyword::For,
            "if" => Keyword::If,
            "or" => Keyword::Or,
            "print" => Keyword::Print,
            "return" => Keyword::Return,
            "super" => Keyword::Super,
            "this" => Keyword::This,
            "var" => Keyword::Var,
            "while" => Keyword::While,
            _ => return None,
        })
    }
}
