use crate::{
    error::{LoxError, LoxErrorKind},
    expression::Expression,
    token::{CodeToken, Symbol, Token, Word},
};

type LoxResult<T> = Result<T, LoxError>;

pub struct Parser {
    current: usize,
    tokens: Vec<CodeToken>,
}

impl Parser {
    pub fn new(tokens: Vec<CodeToken>) -> Parser {
        Parser { current: 0, tokens }
    }

    // Currently parses a single expression - so only a single error can happen
    pub fn parse(&mut self) -> LoxResult<Expression> {
        let expr = self.expression()?;

        // -1 for last index being length-1
        // -1 for last token being unconsumed Eof
        // +1 for current being the next token to be parsed
        if self.current != self.tokens.len() - 1 {
            return Err(LoxError {
                location: self.tokens.last().unwrap().location,
                error_kind: LoxErrorKind::UnparsedTokensLeft,
            });
        } else {
            Ok(expr)
        }
    }

    // Non terminal rules

    fn expression(&mut self) -> LoxResult<Expression> {
        self.equality()
    }

    fn equality(&mut self) -> LoxResult<Expression> {
        let mut expr = self.comparison()?;

        // Short-circuiting is important here for correctness - otherwise this could match `!= ==`
        // as one token
        while self.matches(Token::Symbol(Symbol::BangEqual))
            || self.matches(Token::Symbol(Symbol::EqualEqual))
        {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expression::Binary(Box::new(expr), operator.token, Box::new(right));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> LoxResult<Expression> {
        let mut expr = self.term()?;

        while self.matches(Token::Symbol(Symbol::Greater))
            || self.matches(Token::Symbol(Symbol::GreaterEqual))
            || self.matches(Token::Symbol(Symbol::Less))
            || self.matches(Token::Symbol(Symbol::LessEqual))
        {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expression::Binary(Box::new(expr), operator.token, Box::new(right));
        }

        Ok(expr)
    }

    fn term(&mut self) -> LoxResult<Expression> {
        let mut expr = self.factor()?;

        while self.matches(Token::Symbol(Symbol::Minus))
            || self.matches(Token::Symbol(Symbol::Plus))
        {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expression::Binary(Box::new(expr), operator.token, Box::new(right));
        }

        Ok(expr)
    }

    fn factor(&mut self) -> LoxResult<Expression> {
        let mut expr = self.unary()?;

        while self.matches(Token::Symbol(Symbol::Slash))
            || self.matches(Token::Symbol(Symbol::Star))
        {
            println!("here");
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expression::Binary(Box::new(expr), operator.token, Box::new(right));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> LoxResult<Expression> {
        if self.matches(Token::Symbol(Symbol::Bang)) || self.matches(Token::Symbol(Symbol::Minus)) {
            let operator = self.previous();
            let right = self.unary()?;
            Ok(Expression::Unary(operator.token, Box::new(right)))
        } else {
            Ok(self.primary()?)
        }
    }

    fn primary(&mut self) -> LoxResult<Expression> {
        if let CodeToken {
            token: Token::Literal(l),
            ..
        } = &self.tokens[self.current]
        {
            // manual advance() as part of the manual matches()
            self.current += 1;
            Ok(Expression::Literal(l.clone()))
        } else if self.matches(Token::Symbol(Symbol::LeftParen)) {
            let expr = self.expression()?;
            self.consume(Token::Symbol(Symbol::RightParen))?;
            Ok(Expression::Grouping(Box::new(expr)))
        } else {
            Err(LoxError {
                location: self.tokens[self.current].location,
                error_kind: LoxErrorKind::InvalidExpression,
            })
        }
    }

    fn consume(&mut self, expected: Token) -> LoxResult<CodeToken> {
        if self.check(expected.clone()) {
            Ok(self.advance())
        } else {
            let actual = self.peek();
            Err(LoxError {
                location: actual.location,
                error_kind: LoxErrorKind::UnexpectedToken(actual.token, expected),
            })
        }
    }

    // General utils

    /// This will only advance if the token does match
    fn matches(&mut self, token: Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, token: Token) -> bool {
        if self.is_at_end() {
            return false;
        };
        self.peek().token == token
    }

    fn advance(&mut self) -> CodeToken {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token == Token::Eof
    }

    // Peek doesn't return an option because the last token will
    // always be Token::Eof, so the code should avoid stepping after it
    fn peek(&self) -> CodeToken {
        self.tokens
            .get(self.current)
            .expect("peeked after Token::Eof")
            .clone()
    }

    fn previous(&self) -> CodeToken {
        self.tokens
            .get(self.current - 1)
            .expect("called previous() when current=0")
            .clone()
    }

    // Error handling
    fn synchronize(&mut self) {
        self.advance();

        // Try to drop tokens until we found a statement boundary
        while !self.is_at_end() {
            if self.previous().token == Token::Symbol(Symbol::Semicolon)
                || matches!(self.peek().token, Token::Word(Word::Keyword(k)) if k.is_statement_start())
            {
                return;
            } else {
                self.advance();
            }
        }
    }
}
