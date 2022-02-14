use crate::token::{Literal, Symbol, Token};

#[derive(Debug)]
pub enum Expression {
    Binary(Box<Expression>, BinaryOperator, Box<Expression>),
    Grouping(Box<Expression>),
    Literal(Literal),
    Unary(UnaryOperator, Box<Expression>),
}

#[derive(Debug)]
pub enum UnaryOperator {
    Minus,
    Not,
}

impl UnaryOperator {
    pub fn from_token(token: Token) -> Option<Self> {
        match token {
            Token::Symbol(Symbol::Bang) => Some(UnaryOperator::Not),
            Token::Symbol(Symbol::Minus) => Some(UnaryOperator::Minus),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Divide,
    Multiply,
    Equals,
    NotEquals,
    Less,
    LessEquals,
    Greater,
    GreaterEquals,
}

impl BinaryOperator {
    pub fn from_token(token: Token) -> Option<Self> {
        match token {
            Token::Symbol(Symbol::Plus) => Some(BinaryOperator::Add),
            Token::Symbol(Symbol::Minus) => Some(BinaryOperator::Subtract),
            Token::Symbol(Symbol::Slash) => Some(BinaryOperator::Divide),
            Token::Symbol(Symbol::Star) => Some(BinaryOperator::Multiply),
            Token::Symbol(Symbol::EqualEqual) => Some(BinaryOperator::Equals),
            Token::Symbol(Symbol::BangEqual) => Some(BinaryOperator::NotEquals),
            Token::Symbol(Symbol::Less) => Some(BinaryOperator::Less),
            Token::Symbol(Symbol::LessEqual) => Some(BinaryOperator::LessEquals),
            Token::Symbol(Symbol::Greater) => Some(BinaryOperator::Greater),
            Token::Symbol(Symbol::GreaterEqual) => Some(BinaryOperator::GreaterEquals),
            _ => None,
        }
    }
}
