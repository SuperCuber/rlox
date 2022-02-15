use crate::{
    ast::{BinaryOperator, Expression, Statement, UnaryOperator},
    error::{Located, ParseError, ParseErrorKind},
    token::{CodeToken, Keyword, Symbol, Token},
};

type ParseResult<T> = Result<T, ParseError>;

pub struct Parser {
    current: usize,
    tokens: Vec<CodeToken>,
    errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(tokens: Vec<CodeToken>) -> Parser {
        Parser {
            current: 0,
            tokens,
            errors: Vec::new(),
        }
    }

    // Currently parses a single expression - so only a single error can happen
    pub fn parse(mut self) -> Result<Vec<Statement>, Vec<ParseError>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            if let Some(d) = self.declaration() {
                statements.push(d);
            }
        }
        if self.errors.is_empty() {
            Ok(statements)
        } else {
            Err(self.errors)
        }
    }

    // Non terminal rules

    /// Returns none if there was a parsing error and a synchronization has been run
    fn declaration(&mut self) -> Option<Statement> {
        let statement = if self.matches(Token::Keyword(Keyword::Var)) {
            self.var_declaration()
        } else {
            self.statement()
        };

        match statement {
            Ok(d) => Some(d),
            Err(e) => {
                self.errors.push(e);
                self.synchronize();
                None
            }
        }
    }

    fn var_declaration(&mut self) -> ParseResult<Statement> {
        let name = self.consume_identifier()?;

        let initializer = if self.matches(Token::Symbol(Symbol::Equal)) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(Token::Symbol(Symbol::Semicolon))?;
        Ok(Statement::Var(name.value, initializer))
    }

    fn statement(&mut self) -> ParseResult<Statement> {
        if self.matches(Token::Keyword(Keyword::Print)) {
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> ParseResult<Statement> {
        // Keyword::Print token is already consumed
        let value = self.expression()?;
        self.consume(Token::Symbol(Symbol::Semicolon))?;
        Ok(Statement::Print(value))
    }

    fn expression_statement(&mut self) -> ParseResult<Statement> {
        let expr = self.expression()?;
        self.consume(Token::Symbol(Symbol::Semicolon))?;
        Ok(Statement::Expression(expr))
    }

    fn expression(&mut self) -> ParseResult<Expression> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParseResult<Expression> {
        // Dirty trick: parse lvalue as rvalue
        let expr = self.equality()?;

        if self.matches(Token::Symbol(Symbol::Equal)) {
            let equals = self.previous();
            let value = self.assignment()?;

            // Dirty trick continuation: turn rvalue into an lvalue
            match expr {
                Expression::Variable(v) => Ok(Expression::Assign(v, Box::new(value))),
                _ => {
                    self.errors.push(ParseError {
                        location: equals.location,
                        value: ParseErrorKind::InvalidLvalue,
                    });
                    // Return the lhs expression, ignoring rhs
                    // this is fine because we pushed an error
                    Ok(expr)
                }
            }
        } else {
            Ok(expr)
        }
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
                Located {
                    location: operator.location,
                    value: BinaryOperator::from_token(operator.token).unwrap(),
                },
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
                Located {
                    location: operator.location,
                    value: BinaryOperator::from_token(operator.token).unwrap(),
                },
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
                Located {
                    location: operator.location,
                    value: BinaryOperator::from_token(operator.token).unwrap(),
                },
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
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expression::Binary(
                Box::new(expr),
                Located {
                    location: operator.location,
                    value: BinaryOperator::from_token(operator.token).unwrap(),
                },
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
                Located {
                    location: operator.location,
                    value: UnaryOperator::from_token(operator.token).unwrap(),
                },
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
        } else if let Ok(identifier) = self.consume_identifier() {
            Ok(Expression::Variable(identifier))
        } else if self.matches(Token::Symbol(Symbol::LeftParen)) {
            let expr = self.expression()?;
            self.consume(Token::Symbol(Symbol::RightParen))?;
            Ok(Expression::Grouping(Box::new(expr)))
        } else {
            Err(ParseError {
                location: self.tokens[self.current].location,
                value: ParseErrorKind::InvalidExpression,
            })
        }
    }

    fn consume_identifier(&mut self) -> ParseResult<Located<String>> {
        let actual = self.peek();
        match actual.token {
            Token::Identifier(i) => {
                self.advance();
                Ok(Located {
                    location: actual.location,
                    value: i,
                })
            }
            _ => Err(ParseError {
                location: actual.location,
                value: ParseErrorKind::UnexpectedToken(
                    actual.token,
                    // TODO oh no
                    Token::Identifier("".into()),
                ),
            }),
        }
    }

    fn consume(&mut self, expected: Token) -> ParseResult<CodeToken> {
        if self.check(expected.clone()) {
            Ok(self.advance())
        } else {
            let actual = self.peek();
            Err(ParseError {
                location: actual.location,
                value: ParseErrorKind::UnexpectedToken(actual.token, expected),
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

    // Error recovery
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
