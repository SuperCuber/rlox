use crate::{
    error::Located,
    token::{Literal, Symbol, Token},
};

// Expressions

#[derive(Debug, Clone)]
pub enum Expression {
    Binary(Box<Expression>, Located<BinaryOperator>, Box<Expression>),
    Grouping(Box<Expression>),
    Literal(Literal),
    Unary(Located<UnaryOperator>, Box<Expression>),
    Variable(Located<String>),
    // TODO: I dont like assignment being an expression. I want it to be a statement.
    Assign(Located<String>, Box<Expression>),
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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
    And,
    Or,
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

// Statements
#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Expression),
    Print(Expression),
    Var(String, Option<Expression>),
    While(Expression, Box<Statement>),
    Block(Vec<Statement>),
    If(Expression, Box<Statement>, Option<Box<Statement>>),
}
