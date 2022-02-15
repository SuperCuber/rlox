use crate::{
    error::{TokenizeError, TokenizeErrorKind},
    token::{CodeToken, Keyword, Literal, Symbol, Token},
};

type TokenizeResult<T> = Result<T, TokenizeError>;

// TODO: this is just a function with a fake mustache, refactor it
pub struct Scanner {
    source: String,
    lexeme_start: usize,
    lexeme_len: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            lexeme_start: 0,
            lexeme_len: 0,
            line: 1,
        }
    }

    pub fn tokens(&mut self) -> (Vec<CodeToken>, Vec<TokenizeError>) {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();

        while !self.is_at_end() {
            self.lexeme_start += self.lexeme_len;
            self.lexeme_len = 0;

            let token = self.scan_token();
            let lexeme = self
                .source
                .chars()
                .skip(self.lexeme_start)
                .take(self.lexeme_len)
                .collect();
            match token {
                Ok(token) => {
                    // Sometimes scan_token returns without giving us a token
                    if let Some(token) = token {
                        tokens.push(CodeToken {
                            token,
                            location: self.location(),
                            lexeme,
                        });
                    }
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }

        tokens.push(CodeToken {
            token: Token::Eof,
            location: self.location(),
            lexeme: "".into(),
        });

        (tokens, errors)
    }

    fn scan_token(&mut self) -> TokenizeResult<Option<Token>> {
        Ok(Some(match self.advance() {
            '(' => Token::Symbol(Symbol::LeftParen),
            ')' => Token::Symbol(Symbol::RightParen),
            '{' => Token::Symbol(Symbol::LeftBrace),
            '}' => Token::Symbol(Symbol::RightBrace),
            ',' => Token::Symbol(Symbol::Comma),
            '.' => Token::Symbol(Symbol::Dot),
            '-' => Token::Symbol(Symbol::Minus),
            '+' => Token::Symbol(Symbol::Plus),
            ';' => Token::Symbol(Symbol::Semicolon),
            '*' => Token::Symbol(Symbol::Star),

            // 2-character
            '!' => Token::Symbol(if self.matches('=') {
                Symbol::BangEqual
            } else {
                Symbol::Bang
            }),
            '=' => Token::Symbol(if self.matches('=') {
                Symbol::EqualEqual
            } else {
                Symbol::Equal
            }),
            '<' => Token::Symbol(if self.matches('=') {
                Symbol::LessEqual
            } else {
                Symbol::Less
            }),
            '>' => Token::Symbol(if self.matches('=') {
                Symbol::GreaterEqual
            } else {
                Symbol::Greater
            }),

            // Comments
            '/' => {
                if self.matches('/') {
                    // Comment - go till the end of the line
                    while self.peek() != Some('\n') && self.peek() != None {
                        self.advance();
                    }
                    return Ok(None);
                } else {
                    Token::Symbol(Symbol::Slash)
                }
            }

            // Whitespace
            ' ' | '\r' | '\t' => return Ok(None),
            '\n' => {
                self.line += 1;
                return Ok(None);
            }

            // Literals
            '"' => Token::Literal(Literal::String(self.string()?)),

            c if c.is_ascii_digit() => Token::Literal(Literal::Number(self.number())),

            c if c.is_ascii_alphabetic() || c == '_' => self.word(),

            c => {
                return Err(TokenizeError {
                    location: self.location(),
                    value: TokenizeErrorKind::InvalidStartOfToken(c),
                })
            }
        }))
    }

    // General helpers

    fn peek(&self) -> Option<char> {
        self.source.chars().nth(self.lexeme_start + self.lexeme_len)
    }

    fn is_at_end(&self) -> bool {
        self.peek().is_none()
    }

    fn advance(&mut self) -> char {
        let c = self.peek().expect("advaned past the end of the source");
        self.lexeme_len += 1;
        c
    }

    fn matches(&mut self, expected: char) -> bool {
        if matches!(self.peek(), Some(e) if e == expected) {
            self.lexeme_len += 1;
            true
        } else {
            false
        }
    }

    fn peek_next(&self) -> Option<char> {
        self.source
            .chars()
            .nth(self.lexeme_start + self.lexeme_len + 1)
    }

    fn location(&self) -> (usize, usize) {
        let lexeme_start_byte = self.source.char_indices().nth(self.lexeme_start).unwrap().0;
        let before_current = &self.source[0..lexeme_start_byte];
        // line is different than self.line in case of multiline lexeme (like a string)
        let line = before_current.chars().filter(|c| *c == '\n').count();
        let last_line_start = before_current.rfind('\n').map(|x| x + 1).unwrap_or(0);
        // + 1 for 1-indexed
        (line + 1, self.lexeme_start - last_line_start + 1)
    }

    // Token helpers

    fn string(&mut self) -> TokenizeResult<String> {
        while self.peek() != Some('"') && self.peek() != None {
            if self.peek().unwrap() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            Err(TokenizeError {
                location: self.location(),
                value: TokenizeErrorKind::UnterminatedString,
            })
        } else {
            // Consume `"`
            self.advance();

            let value = self
                .source
                .chars()
                .skip(self.lexeme_start + 1)
                // -1 for starting quote, -1 for ending quote
                .take(self.lexeme_len - 2)
                .collect();
            Ok(value)
        }
    }

    fn number(&mut self) -> f32 {
        while self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            self.advance();
        }
        if self.peek() == Some('.')
            && self
                .peek_next()
                .map(|n| n.is_ascii_digit())
                .unwrap_or(false)
        {
            // Consume `.`
            self.advance();

            while self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                self.advance();
            }
        }

        let number: String = self
            .source
            .chars()
            .skip(self.lexeme_start)
            .take(self.lexeme_len)
            .collect();

        number.parse().unwrap()
    }

    fn word(&mut self) -> Token {
        while self
            .peek()
            .map(|c| c.is_ascii_alphanumeric())
            .unwrap_or(false)
        {
            self.advance();
        }

        let text: String = self
            .source
            .chars()
            .skip(self.lexeme_start)
            .take(self.lexeme_len)
            .collect();

        if let Some(keyword) = Keyword::from_word(&text) {
            Token::Keyword(keyword)
        } else if text == "false" {
            Token::Literal(Literal::Boolean(false))
        } else if text == "true" {
            Token::Literal(Literal::Boolean(true))
        } else if text == "nil" {
            Token::Literal(Literal::Nil)
        } else {
            Token::Identifier(text)
        }
    }
}
