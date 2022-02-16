use crate::{
    ast::{BinaryOperator, CodeExpression, Expression, Statement, UnaryOperator},
    error::{Located, ParseError, ParseErrorKind},
    token::{CodeToken, Keyword, Literal, Symbol, Token},
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

    pub fn parse_expression(mut self) -> Result<CodeExpression, Vec<ParseError>> {
        let expression = self.expression().map_err(|e| vec![e])?;
        if self.errors.is_empty() {
            if self.is_at_end() {
                Ok(expression)
            } else {
                // Errors are thrown away anyways
                Err(vec![])
            }
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
        if self.matches(Token::Keyword(Keyword::If)) {
            self.if_statement()
        } else if self.matches(Token::Keyword(Keyword::For)) {
            self.for_statement()
        } else if self.matches(Token::Keyword(Keyword::While)) {
            self.while_statement()
        } else if self.matches(Token::Keyword(Keyword::Print)) {
            self.print_statement()
        } else if self.matches(Token::Symbol(Symbol::LeftBrace)) {
            self.block().map(Statement::Block)
        } else {
            self.expression_statement()
        }
    }

    fn if_statement(&mut self) -> ParseResult<Statement> {
        self.consume(Token::Symbol(Symbol::LeftParen))?;
        let condition = self.expression()?;
        self.consume(Token::Symbol(Symbol::RightParen))?;

        let then_branch = self.statement()?;
        let else_branch = if self.matches(Token::Keyword(Keyword::Else)) {
            Some(self.statement()?)
        } else {
            None
        };

        Ok(Statement::If(
            condition,
            Box::new(then_branch),
            else_branch.map(Box::new),
        ))
    }

    /// Desugared into a while loop
    fn for_statement(&mut self) -> ParseResult<Statement> {
        self.consume(Token::Symbol(Symbol::LeftParen))?;
        let initializer = if self.matches(Token::Symbol(Symbol::Semicolon)) {
            None
        } else if self.matches(Token::Keyword(Keyword::Var)) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if self.check(Token::Symbol(Symbol::Semicolon)) {
            let semicolon = self.consume(Token::Symbol(Symbol::Semicolon)).unwrap();
            CodeExpression {
                location: semicolon.location,
                value: Expression::Literal(Literal::Boolean(true)),
            }
        } else {
            self.consume(Token::Symbol(Symbol::Semicolon))?;
            self.expression()?
        };

        let increment = if !self.check(Token::Symbol(Symbol::RightParen)) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(Token::Symbol(Symbol::RightParen))?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Statement::Block(vec![body, Statement::Expression(increment)]);
        }

        body = Statement::While(condition, Box::new(body));

        if let Some(initializer) = initializer {
            body = Statement::Block(vec![initializer, body]);
        }

        Ok(body)
    }

    fn while_statement(&mut self) -> ParseResult<Statement> {
        self.consume(Token::Symbol(Symbol::LeftParen))?;
        let condition = self.expression()?;
        self.consume(Token::Symbol(Symbol::RightParen))?;
        let body = self.statement()?;

        Ok(Statement::While(condition, Box::new(body)))
    }

    fn print_statement(&mut self) -> ParseResult<Statement> {
        // Keyword::Print token is already consumed
        let value = self.expression()?;
        self.consume(Token::Symbol(Symbol::Semicolon))?;
        Ok(Statement::Print(value))
    }

    // Doesn't return a Statement::Block directly for reusability in function parsing
    fn block(&mut self) -> ParseResult<Vec<Statement>> {
        // Left brace already consumed
        let mut statements = Vec::new();
        while !self.check(Token::Symbol(Symbol::RightBrace)) && !self.is_at_end() {
            if let Some(d) = self.declaration() {
                statements.push(d);
            }
        }
        self.consume(Token::Symbol(Symbol::RightBrace))?;
        Ok(statements)
    }

    fn expression_statement(&mut self) -> ParseResult<Statement> {
        let expr = self.expression()?;
        self.consume(Token::Symbol(Symbol::Semicolon))?;
        Ok(Statement::Expression(expr))
    }

    fn expression(&mut self) -> ParseResult<CodeExpression> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParseResult<CodeExpression> {
        // Dirty trick: parse lvalue as rvalue
        let expr = self.or()?;

        if self.matches(Token::Symbol(Symbol::Equal)) {
            let equals = self.previous();
            let value = self.assignment()?;

            // Dirty trick continuation: turn rvalue into an lvalue
            match expr.value {
                Expression::Variable(v) => Ok(CodeExpression {
                    location: equals.location,
                    value: Expression::Assign(v, Box::new(value)),
                }),
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

    fn or(&mut self) -> ParseResult<CodeExpression> {
        let mut expr = self.and()?;

        while self.matches(Token::Keyword(Keyword::Or)) {
            let operator = self.previous();
            let right = self.and()?;
            expr = CodeExpression {
                location: operator.location,
                value: Expression::Binary(Box::new(expr), BinaryOperator::Or, Box::new(right)),
            };
        }

        Ok(expr)
    }

    fn and(&mut self) -> ParseResult<CodeExpression> {
        let mut expr = self.equality()?;

        while self.matches(Token::Keyword(Keyword::And)) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = CodeExpression {
                location: operator.location,
                value: Expression::Binary(Box::new(expr), BinaryOperator::And, Box::new(right)),
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> ParseResult<CodeExpression> {
        let mut expr = self.comparison()?;

        // Short-circuiting is important here for correctness - otherwise this could match `!= ==`
        // as one token
        while self.matches(Token::Symbol(Symbol::BangEqual))
            || self.matches(Token::Symbol(Symbol::EqualEqual))
        {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = CodeExpression {
                location: operator.location,
                value: Expression::Binary(
                    Box::new(expr),
                    BinaryOperator::from_token(operator.token).unwrap(),
                    Box::new(right),
                ),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> ParseResult<CodeExpression> {
        let mut expr = self.term()?;

        while self.matches(Token::Symbol(Symbol::Greater))
            || self.matches(Token::Symbol(Symbol::GreaterEqual))
            || self.matches(Token::Symbol(Symbol::Less))
            || self.matches(Token::Symbol(Symbol::LessEqual))
        {
            let operator = self.previous();
            let right = self.term()?;
            expr = CodeExpression {
                location: operator.location,
                value: Expression::Binary(
                    Box::new(expr),
                    BinaryOperator::from_token(operator.token).unwrap(),
                    Box::new(right),
                ),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> ParseResult<CodeExpression> {
        let mut expr = self.factor()?;

        while self.matches(Token::Symbol(Symbol::Minus))
            || self.matches(Token::Symbol(Symbol::Plus))
        {
            let operator = self.previous();
            let right = self.factor()?;
            expr = CodeExpression {
                location: operator.location,
                value: Expression::Binary(
                    Box::new(expr),
                    BinaryOperator::from_token(operator.token).unwrap(),
                    Box::new(right),
                ),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> ParseResult<CodeExpression> {
        let mut expr = self.unary()?;

        while self.matches(Token::Symbol(Symbol::Slash))
            || self.matches(Token::Symbol(Symbol::Star))
        {
            let operator = self.previous();
            let right = self.unary()?;
            expr = CodeExpression {
                location: operator.location,
                value: Expression::Binary(
                    Box::new(expr),
                    BinaryOperator::from_token(operator.token).unwrap(),
                    Box::new(right),
                ),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> ParseResult<CodeExpression> {
        if self.matches(Token::Symbol(Symbol::Bang)) || self.matches(Token::Symbol(Symbol::Minus)) {
            let operator = self.previous();
            let right = self.unary()?;
            Ok(CodeExpression {
                location: operator.location,
                value: Expression::Unary(
                    UnaryOperator::from_token(operator.token).unwrap(),
                    Box::new(right),
                ),
            })
        } else {
            Ok(self.primary()?)
        }
    }

    fn primary(&mut self) -> ParseResult<CodeExpression> {
        if let CodeToken {
            token: Token::Literal(l),
            location,
            ..
        } = &self.tokens[self.current]
        {
            // manual advance() as part of the manual matches()
            self.current += 1;
            Ok(CodeExpression {
                location: *location,
                value: Expression::Literal(l.clone()),
            })
        } else if let Ok(identifier) = self.consume_identifier() {
            Ok(CodeExpression {
                location: identifier.location,
                value: Expression::Variable(identifier.value),
            })
        } else if let Ok(left_paren) = self.consume(Token::Symbol(Symbol::LeftParen)) {
            let expr = self.expression()?;
            self.consume(Token::Symbol(Symbol::RightParen))?;
            Ok(CodeExpression {
                location: left_paren.location,
                value: Expression::Grouping(Box::new(expr)),
            })
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
