use std::{error::Error, fmt::Display, io};

use crate::{
    token::Token,
    value::{SharedValue, Type},
};

#[derive(Debug, thiserror::Error)]
pub enum LoxError {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Tokenize(#[from] TokenizeError),
    #[error("{0}")]
    Parse(#[from] ParseError),
    #[error("{0}")]
    Runtime(#[from] RuntimeError),
}

#[derive(Debug, Clone)]
pub struct Located<T> {
    pub location: (usize, usize),
    pub value: T,
}

pub type TokenizeError = Located<TokenizeErrorKind>;

#[derive(Debug, thiserror::Error)]
pub enum TokenizeErrorKind {
    #[error("unexpected start of token: `{0}`")]
    InvalidStartOfToken(char),
    #[error("unterminated string")]
    UnterminatedString,
}

pub type ParseError = Located<ParseErrorKind>;

#[derive(Debug, thiserror::Error)]
pub enum ParseErrorKind {
    #[error("unexpected token {0:?}, expected {1:?}")]
    UnexpectedToken(Token, Token),
    #[error("invalid expression")]
    InvalidExpression,
    #[error("invalid assignment target")]
    InvalidLvalue,
    #[error("too many arguments (maximum is {0})")]
    TooManyArguments(usize),
}

pub type RuntimeError = Located<RuntimeErrorKind>;

#[derive(Debug, thiserror::Error)]
pub enum RuntimeErrorKind {
    #[error("expected type {0:?}, got {1:?}")]
    TypeError(Type, Type),
    #[error("expected types {0:?}, got {1:?}")]
    TypeErrorMultiple(Vec<Type>, Type),
    #[error("undefined variable `{0}`")]
    UndefinedVariable(String),
    #[error("wrong number of arguments: got {0}, expected {1}")]
    WrongArgsNum(usize, usize),

    /// not actually an error
    #[error("RETURNING, YOU SHOULD NEVER SEE THIS")]
    Returning(SharedValue),
}

impl<E: Error> Display for Located<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}:{}] Error: {}",
            self.location.0, self.location.1, self.value
        )
    }
}

impl<E: Error> Error for Located<E> {}

pub trait WithLocation {
    type Output;
    fn with_location(self, location: (usize, usize)) -> Self::Output;
}

impl<T, E> WithLocation for Result<T, E> {
    type Output = Result<T, Located<E>>;
    fn with_location(self, location: (usize, usize)) -> Result<T, Located<E>> {
        self.map_err(|e| Located { location, value: e })
    }
}
