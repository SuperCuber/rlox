use std::{error::Error, fmt::Display};

use crate::{token::Token, value::Type};

#[derive(Debug)]
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
}

pub type RuntimeError = Located<RuntimeErrorKind>;

#[derive(Debug, thiserror::Error)]
pub enum RuntimeErrorKind {
    #[error("invalid expression")]
    TypeError(Type, Type),
    #[error("invalid expression")]
    TypeErrorMultiple(Vec<Type>, Type),
    #[error("invalid expression")]
    UndefinedVariable(String),
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
