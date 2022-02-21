pub struct Scanner {
    source: String,
    lexeme_start: usize,
    current_char: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            lexeme_start: 0,
            current_char: 0,
            line: 1,
        }
    }

    pub fn scan_token(&mut self) -> Result<CodeToken, ScanError> {
        self.skip_whitespace();
        self.lexeme_start = self.current_char;

        if self.is_at_end() {
            return Ok(self.make_token(Token::Eof));
        }

        let c = self.advance();

        Ok(match c {
            '(' => self.make_token(Token::LeftParen),
            ')' => self.make_token(Token::RightParen),
            '{' => self.make_token(Token::LeftBrace),
            '}' => self.make_token(Token::RightBrace),
            ';' => self.make_token(Token::Semicolon),
            ',' => self.make_token(Token::Comma),
            '.' => self.make_token(Token::Dot),
            '-' => self.make_token(Token::Minus),
            '+' => self.make_token(Token::Plus),
            '/' => self.make_token(Token::Slash),
            '*' => self.make_token(Token::Star),
            '!' => {
                let matches = self.matches('=');
                self.make_token(if matches {
                    Token::BangEqual
                } else {
                    Token::Bang
                })
            }
            '=' => {
                let matches = self.matches('=');
                self.make_token(if matches {
                    Token::EqualEqual
                } else {
                    Token::Equal
                })
            }
            '<' => {
                let matches = self.matches('=');
                self.make_token(if matches {
                    Token::LessEqual
                } else {
                    Token::Less
                })
            }
            '>' => {
                let matches = self.matches('=');
                self.make_token(if matches {
                    Token::GreaterEqual
                } else {
                    Token::Greater
                })
            }
            '"' => {
                let s = self.string()?;
                self.make_token(Token::Literal(Literal::String(s)))
            }
            d if d.is_ascii_digit() => {
                let n = self.number(d);
                self.make_token(Token::Literal(Literal::Number(n)))
            }
            l if l.is_ascii_alphabetic() || l == '_' => {
                let i = self.identifier(l);
                self.make_token(i)
            }
            _ => {
                return Err(ScanError {
                    line: self.line,
                    error: "Unexpected character.",
                })
            }
        })
    }

    fn skip_whitespace(&mut self) {
        loop {
            match self.peek() {
                // Skip comments
                Some('/') if self.peek_next() == Some('/') => {
                    while self.peek() != Some('\n') && !self.is_at_end() {
                        self.advance();
                    }
                }
                // Count newlines
                Some('\n') => {
                    self.line += 1;
                    self.advance();
                }
                // Skip other kinds of whitespace
                Some(' ' | '\r' | '\t') => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    fn string(&mut self) -> Result<String, ScanError> {
        let mut string = String::new();
        while self.peek() != Some('"') && !self.is_at_end() {
            if self.peek() == Some('\n') {
                self.line += 1;
            }
            string.push(self.advance());
        }

        if self.is_at_end() {
            Err(ScanError {
                line: self.line,
                error: "Unterminated string.",
            })
        } else {
            // closing quote
            self.advance();
            Ok(string)
        }
    }

    fn number(&mut self, d: char) -> f32 {
        let mut number = String::from(d);
        while self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            number.push(self.advance());
        }
        if self.peek() == Some('.')
            && self
                .peek_next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            // dot
            number.push(self.advance());

            while self.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                number.push(self.advance());
            }
        }

        number.parse().unwrap()
    }

    fn identifier(&mut self, l: char) -> Token {
        let mut i = String::from(l);
        while self
            .peek()
            .map(|c| c.is_ascii_alphabetic() || c == '_')
            .unwrap_or(false)
        {
            i.push(self.advance());
        }
        if let Some(keyword) = Token::from_keyword(&i) {
            keyword
        } else {
            Token::Identifier(i)
        }
    }

    // Util

    fn is_at_end(&self) -> bool {
        self.peek().is_none()
    }

    fn peek(&self) -> Option<char> {
        self.source.chars().nth(self.current_char)
    }

    fn peek_next(&self) -> Option<char> {
        self.source.chars().nth(self.current_char + 1)
    }

    fn matches(&mut self, c: char) -> bool {
        let ans = self.peek() == Some(c);
        if ans {
            self.current_char += 1;
        }
        ans
    }

    fn advance(&mut self) -> char {
        self.current_char += 1;
        self.source
            .chars()
            .nth(self.current_char - 1)
            .expect("advanced past end")
    }

    fn make_token(&self, token: Token) -> CodeToken {
        CodeToken {
            token,
            line: self.line,
            lexeme: self
                .source
                .chars()
                .skip(self.lexeme_start)
                .take(self.current_char - self.lexeme_start)
                .collect(),
        }
    }
}

pub struct ScanError {
    pub line: usize,
    pub error: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CodeToken {
    pub token: Token,
    pub line: usize,
    pub lexeme: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Literal(Literal),
    Identifier(String),

    // Keywords
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

    // Symbols
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

    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Number(f32),
    Boolean(bool),
    Nil,
}

impl Token {
    pub fn from_keyword(word: &str) -> Option<Token> {
        Some(match word {
            "and" => Token::And,
            "class" => Token::Class,
            "else" => Token::Else,
            "fun" => Token::Fun,
            "for" => Token::For,
            "if" => Token::If,
            "or" => Token::Or,
            "print" => Token::Print,
            "return" => Token::Return,
            "super" => Token::Super,
            "this" => Token::This,
            "var" => Token::Var,
            "while" => Token::While,
            _ => return None,
        })
    }

    pub fn is_statement_start(&self) -> bool {
        matches!(
            self,
            Token::Class
                | Token::Fun
                | Token::Var
                | Token::For
                | Token::If
                | Token::While
                | Token::Print
                | Token::Return
        )
    }
}
