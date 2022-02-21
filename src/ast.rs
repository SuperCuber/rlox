use crate::{
    error::Located,
    token::{Literal, Symbol, Token},
};

// Expressions

pub type GenericCodeExpression<V> = Located<Expression<V>>;
pub type CodeExpression = Located<Expression<String>>;
pub type ResolvedCodeExpression = Located<Expression<ResolvedVariable>>;

#[derive(Debug, Clone)]
pub enum Expression<V> {
    Binary(
        Box<GenericCodeExpression<V>>,
        BinaryOperator,
        Box<GenericCodeExpression<V>>,
    ),
    Call(Box<GenericCodeExpression<V>>, Vec<GenericCodeExpression<V>>),
    Grouping(Box<GenericCodeExpression<V>>),
    Literal(Literal),
    Unary(UnaryOperator, Box<GenericCodeExpression<V>>),
    Variable(V),
    // TODO: I dont like assignment being an expression. I want it to be a statement.
    Assign(V, Box<GenericCodeExpression<V>>),
}

#[derive(Clone)]
pub struct ResolvedVariable {
    pub name: String,
    /// none = global
    pub hops: Option<usize>,
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

pub type Statement = GenericStatement<String>;
pub type ResolvedStatement = GenericStatement<ResolvedVariable>;

// Statements
#[derive(Debug, Clone)]
pub enum GenericStatement<V> {
    Expression(GenericCodeExpression<V>),
    Function(String, Vec<String>, Vec<GenericStatement<V>>),
    Print(GenericCodeExpression<V>),
    Return(Option<GenericCodeExpression<V>>),
    Var(String, Option<GenericCodeExpression<V>>),
    While(GenericCodeExpression<V>, Box<GenericStatement<V>>),
    Block(Vec<GenericStatement<V>>),
    If(
        GenericCodeExpression<V>,
        Box<GenericStatement<V>>,
        Option<Box<GenericStatement<V>>>,
    ),
}
