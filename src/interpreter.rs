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

    fn evaluate(&self, expression: Expression) -> RuntimeResult<Value> {
        match expression {
            Expression::Literal(l) => Ok(self.evaluate_literal(l)),
            Expression::Grouping(e) => self.evaluate_grouping(*e),
            Expression::Unary(o, r) => self.evaluate_unary(o, *r),
            Expression::Binary(l, o, r) => self.evaluate_binary(*l, o, *r),
            Expression::Variable(v) => self.environment.get(v.value).with_location(v.location),
        }
    }

    fn evaluate_literal(&self, literal: Literal) -> Value {
        match literal {
            Literal::String(s) => Value::String(s),
            Literal::Number(n) => Value::Number(n),
            Literal::Boolean(b) => Value::Boolean(b),
            Literal::Nil => Value::Nil,
        }
    }

    fn evaluate_grouping(&self, e: Expression) -> RuntimeResult<Value> {
        self.evaluate(e)
    }

    fn evaluate_unary(&self, o: Located<UnaryOperator>, r: Expression) -> RuntimeResult<Value> {
        let right = self.evaluate(r)?;

        Ok(match o.value {
            UnaryOperator::Minus => Value::Number(-right.into_number().with_location(o.location)?),
            UnaryOperator::Not => Value::Boolean(!right.into_boolean().with_location(o.location)?),
        })
    }

    fn evaluate_binary(
        &self,
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
