use crate::{
    ast::{BinaryOperator, Expression, Statement, UnaryOperator},
    environment::Environment,
    error::{Located, RuntimeError, RuntimeErrorKind, WithLocation},
    token::Literal,
    value::{Type, Value},
};

pub type RuntimeResult<T> = Result<T, RuntimeError>;

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            environment: Environment::new(),
        }
    }

    pub fn interpret(&mut self, program: Vec<Statement>) -> RuntimeResult<()> {
        for statement in program {
            self.execute(statement)?;
        }
        Ok(())
    }

    fn execute(&mut self, statement: Statement) -> RuntimeResult<()> {
        match statement {
            Statement::Expression(expr) => self.evaluate(expr).map(|_| ()),
            Statement::Print(expr) => {
                println!("{}", self.evaluate(expr)?);
                Ok(())
            }
            Statement::Var(name, value) => self.execute_statement_var(name, value),
            Statement::Block(b) => self.execute_block(b),
            Statement::If(condition, then_branch, else_branch) => {
                self.execute_if(condition, *then_branch, else_branch)
            }
            Statement::While(condition, body) => self.execute_while(condition, *body),
        }
    }

    fn execute_statement_var(
        &mut self,
        name: String,
        value: Option<Expression>,
    ) -> Result<(), RuntimeError> {
        let value = if let Some(e) = value {
            self.evaluate(e)?
        } else {
            Value::Nil
        };
        self.environment.define(name, value);

        Ok(())
    }

    fn execute_block(&mut self, statements: Vec<Statement>) -> RuntimeResult<()> {
        self.environment.push_env();
        for statement in statements {
            if let Err(e) = self.execute(statement) {
                self.environment.pop_env();
                return Err(e);
            }
        }
        self.environment.pop_env();
        Ok(())
    }

    fn execute_if(
        &mut self,
        condition: Expression,
        then_branch: Statement,
        else_branch: Option<Box<Statement>>,
    ) -> RuntimeResult<()> {
        let condition = self.evaluate(condition)?;
        // TODO oh no
        if condition.into_boolean().with_location((0, 0))? {
            self.execute(then_branch)?;
        } else if let Some(else_branch) = else_branch {
            self.execute(*else_branch)?;
        }
        Ok(())
    }

    fn execute_while(&mut self, condition: Expression, body: Statement) -> RuntimeResult<()> {
        // TODO oh no
        while self
            .evaluate(condition.clone())?
            .into_boolean()
            .with_location((0, 0))?
        {
            self.execute(body.clone())?;
        }
        Ok(())
    }

    pub fn evaluate(&mut self, expression: Expression) -> RuntimeResult<Value> {
        match expression {
            Expression::Literal(l) => Ok(self.evaluate_literal(l)),
            Expression::Grouping(e) => self.evaluate_grouping(*e),
            Expression::Unary(o, r) => self.evaluate_unary(o, *r),
            Expression::Binary(l, o, r) => self.evaluate_binary(*l, o, *r),
            Expression::Variable(v) => self.environment.get(v.value).with_location(v.location),
            Expression::Assign(v, e) => self.evaluate_assign(v, *e),
        }
    }

    fn evaluate_assign(
        &mut self,
        variable: Located<String>,
        expression: Expression,
    ) -> RuntimeResult<Value> {
        let value = self.evaluate(expression)?;
        self.environment
            .assign(variable.value, value.clone())
            .with_location(variable.location)?;
        Ok(value)
    }

    fn evaluate_literal(&self, literal: Literal) -> Value {
        match literal {
            Literal::String(s) => Value::String(s),
            Literal::Number(n) => Value::Number(n),
            Literal::Boolean(b) => Value::Boolean(b),
            Literal::Nil => Value::Nil,
        }
    }

    fn evaluate_grouping(&mut self, e: Expression) -> RuntimeResult<Value> {
        self.evaluate(e)
    }

    fn evaluate_unary(&mut self, o: Located<UnaryOperator>, r: Expression) -> RuntimeResult<Value> {
        let right = self.evaluate(r)?;

        Ok(match o.value {
            UnaryOperator::Minus => Value::Number(-right.into_number().with_location(o.location)?),
            UnaryOperator::Not => Value::Boolean(!right.into_boolean().with_location(o.location)?),
        })
    }

    fn evaluate_binary(
        &mut self,
        left: Expression,
        operator: Located<BinaryOperator>,
        right: Expression,
    ) -> RuntimeResult<Value> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;
        let loc = operator.location;
        Ok(match operator.value {
            // Math
            BinaryOperator::Subtract => Value::Number(
                left.into_number().with_location(loc)? - right.into_number().with_location(loc)?,
            ),
            BinaryOperator::Divide => Value::Number(
                left.into_number().with_location(loc)? / right.into_number().with_location(loc)?,
            ),
            BinaryOperator::Multiply => Value::Number(
                left.into_number().with_location(loc)? * right.into_number().with_location(loc)?,
            ),
            // Comparison
            BinaryOperator::Less => Value::Boolean(
                left.into_number().with_location(loc)? < right.into_number().with_location(loc)?,
            ),
            BinaryOperator::LessEquals => Value::Boolean(
                left.into_number().with_location(loc)? <= right.into_number().with_location(loc)?,
            ),
            BinaryOperator::Greater => Value::Boolean(
                left.into_number().with_location(loc)? > right.into_number().with_location(loc)?,
            ),
            BinaryOperator::GreaterEquals => Value::Boolean(
                left.into_number().with_location(loc)? >= right.into_number().with_location(loc)?,
            ),
            // Equality
            BinaryOperator::Equals => Value::Boolean(left == right),
            BinaryOperator::NotEquals => Value::Boolean(left != right),
            // Logical - short circuiting
            BinaryOperator::And => {
                let left = left.into_boolean().with_location(loc)?;
                let right = right.into_boolean().with_location(loc)?;
                if !left {
                    Value::Boolean(false)
                } else {
                    Value::Boolean(right)
                }
            }
            BinaryOperator::Or => {
                let left = left.into_boolean().with_location(loc)?;
                let right = right.into_boolean().with_location(loc)?;
                if left {
                    Value::Boolean(true)
                } else {
                    Value::Boolean(right)
                }
            }
            // Add
            BinaryOperator::Add => {
                if let Ok(left) = left.clone().into_number().with_location(loc) {
                    let right = right.into_number().with_location(loc)?;
                    Value::Number(left + right)
                } else if let Ok(left) = left.clone().into_string().with_location(loc) {
                    let right = right.into_string().with_location(loc)?;
                    Value::String(left + &right)
                } else {
                    return Err(RuntimeError {
                        location: loc,
                        value: RuntimeErrorKind::TypeErrorMultiple(
                            vec![Type::Number, Type::String],
                            left.value_type(),
                        ),
                    });
                }
            }
        })
    }
}
