use std::fmt::Display;

#[derive(Debug)]
pub struct LoxError {
    pub line: usize,
    pub error_kind: LoxErrorKind,
}

#[derive(Debug, thiserror::Error)]
pub enum LoxErrorKind {
    #[error("unexpected start of token: `{0}`")]
    InvalidStartOfToken(char),
    #[error("unterminated string")]
    UnterminatedString,
}

impl Display for LoxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] Error: {}", self.line, self.error_kind)
    }
}
