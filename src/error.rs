use std::fmt::Display;

use crate::{token::Token, value::Type};

#[derive(Debug, thiserror::Error)]
pub struct TokenizeError {
    pub location: (usize, usize),
    pub error_kind: TokenizeErrorKind,
}

#[derive(Debug, thiserror::Error)]
pub enum TokenizeErrorKind {
    #[error("unexpected start of token: `{0}`")]
    InvalidStartOfToken(char),
    #[error("unterminated string")]
    UnterminatedString,
}

#[derive(Debug, thiserror::Error)]
pub struct ParseError {
    pub location: (usize, usize),
    pub error_kind: ParseErrorKind,
}

#[derive(Debug, thiserror::Error)]
pub enum ParseErrorKind {
    #[error("unexpected token {0:?}, expected {1:?}")]
    UnexpectedToken(Token, Token),
    #[error("invalid expression")]
    InvalidExpression,
    #[error("unparsed tokens left")]
    UnparsedTokensLeft,
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    TypeError((usize, usize), Type, Type),
    TypeErrorMultiple((usize, usize), Vec<Type>, Type),
}

impl Display for TokenizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}:{}] Error: {}",
            self.location.0, self.location.1, self.error_kind
        )
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}:{}] Error: {}",
            self.location.0, self.location.1, self.error_kind
        )
    }
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::TypeError((line, col), expected, actual) => {
                write!(
                    f,
                    "[{line}:{col}] Error: Type mismatch: expected {expected:?}, got {actual:?}"
                )
            }
            RuntimeError::TypeErrorMultiple((line, col), expected, actual) => {
                write!(
                    f,
                    "[{line}:{col}] Error: Type mismatch: expected {expected:?}, got {actual:?}"
                )
            }
        }
    }
}
