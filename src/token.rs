#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub struct CodeToken {
    pub token: Token,
    pub location: (usize, usize),
    pub lexeme: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Literal(Literal),
    Symbol(Symbol),
    Word(Word),
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Word {
    Identifier(String),
    Keyword(Keyword),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Number(f32),
    Boolean,
    Nil,
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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

    pub fn is_statement_start(&self) -> bool {
        matches!(
            self,
            Keyword::Class
                | Keyword::Fun
                | Keyword::Var
                | Keyword::For
                | Keyword::If
                | Keyword::While
                | Keyword::Print
                | Keyword::Return
        )
    }
}
