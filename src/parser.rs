use crate::{
    error::{ParseError, ParseErrorKind},
    expression::{BinaryOperator, Expression, UnaryOperator},
    token::{CodeToken, Symbol, Token},
};

type ParseResult<T> = Result<T, ParseError>;

pub struct Parser {
    current: usize,
    tokens: Vec<CodeToken>,
}

impl Parser {
    pub fn new(tokens: Vec<CodeToken>) -> Parser {
        Parser { current: 0, tokens }
    }

    // Currently parses a single expression - so only a single error can happen
    pub fn parse(&mut self) -> ParseResult<Expression> {
        let expr = self.expression()?;

        // -1 for last index being length-1
        // -1 for last token being unconsumed Eof
        // +1 for current being the next token to be parsed
        if self.current != self.tokens.len() - 1 {
            return Err(ParseError {
                location: self.tokens.last().unwrap().location,
                error_kind: ParseErrorKind::UnparsedTokensLeft,
            });
        } else {
            Ok(expr)
        }
    }

    // Non terminal rules

    fn expression(&mut self) -> ParseResult<Expression> {
        self.equality()
    }

    fn equality(&mut self) -> ParseResult<Expression> {
        let mut expr = self.comparison()?;

        // Short-circuiting is important here for correctness - otherwise this could match `!= ==`
        // as one token
        while self.matches(Token::Symbol(Symbol::BangEqual))
            || self.matches(Token::Symbol(Symbol::EqualEqual))
        {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expression::Binary(
                Box::new(expr),
                BinaryOperator::from_token(operator.token).unwrap(),
                Box::new(right),
            );
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> ParseResult<Expression> {
        let mut expr = self.term()?;

        while self.matches(Token::Symbol(Symbol::Greater))
            || self.matches(Token::Symbol(Symbol::GreaterEqual))
            || self.matches(Token::Symbol(Symbol::Less))
            || self.matches(Token::Symbol(Symbol::LessEqual))
        {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expression::Binary(
                Box::new(expr),
                BinaryOperator::from_token(operator.token).unwrap(),
                Box::new(right),
            );
        }

        Ok(expr)
    }

    fn term(&mut self) -> ParseResult<Expression> {
        let mut expr = self.factor()?;

        while self.matches(Token::Symbol(Symbol::Minus))
            || self.matches(Token::Symbol(Symbol::Plus))
        {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expression::Binary(
                Box::new(expr),
                BinaryOperator::from_token(operator.token).unwrap(),
                Box::new(right),
            );
        }

        Ok(expr)
    }

    fn factor(&mut self) -> ParseResult<Expression> {
        let mut expr = self.unary()?;

        while self.matches(Token::Symbol(Symbol::Slash))
            || self.matches(Token::Symbol(Symbol::Star))
        {
            println!("here");
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expression::Binary(
                Box::new(expr),
                BinaryOperator::from_token(operator.token).unwrap(),
                Box::new(right),
            );
        }

        Ok(expr)
    }

    fn unary(&mut self) -> ParseResult<Expression> {
        if self.matches(Token::Symbol(Symbol::Bang)) || self.matches(Token::Symbol(Symbol::Minus)) {
            let operator = self.previous();
            let right = self.unary()?;
            Ok(Expression::Unary(
                UnaryOperator::from_token(operator.token).unwrap(),
                Box::new(right),
            ))
        } else {
            Ok(self.primary()?)
        }
    }

    fn primary(&mut self) -> ParseResult<Expression> {
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
            Err(ParseError {
                location: self.tokens[self.current].location,
                error_kind: ParseErrorKind::InvalidExpression,
            })
        }
    }

    fn consume(&mut self, expected: Token) -> ParseResult<CodeToken> {
        if self.check(expected.clone()) {
            Ok(self.advance())
        } else {
            let actual = self.peek();
            Err(ParseError {
                location: actual.location,
                error_kind: ParseErrorKind::UnexpectedToken(actual.token, expected),
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
                || matches!(self.peek().token, Token::Keyword(k) if k.is_statement_start())
            {
                return;
            } else {
                self.advance();
            }
        }
    }
}
