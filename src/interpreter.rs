use crate::{
    error::RuntimeError,
    expression::{
        BinaryOperator, CodeBinaryOperator, CodeUnaryOperator, Expression, UnaryOperator,
    },
    token::Literal,
    value::{Type, Value},
};

type RuntimeResult<T> = Result<T, RuntimeError>;

pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter
    }

    pub fn interpret(&mut self, e: Expression) -> Result<(), RuntimeError> {
        let value = self.evaluate(e)?;
        println!("{value}");
        Ok(())
    }

    fn evaluate(&self, expression: Expression) -> RuntimeResult<Value> {
        match expression {
            Expression::Literal(l) => Ok(self.evaluate_literal(l)),
            Expression::Grouping(e) => self.evaluate_grouping(*e),
            Expression::Unary(o, r) => self.evaluate_unary(o, *r),
            Expression::Binary(l, o, r) => self.evaluate_binary(*l, o, *r),
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

    fn evaluate_unary(&self, o: CodeUnaryOperator, r: Expression) -> RuntimeResult<Value> {
        let right = self.evaluate(r)?;

        Ok(match o.op {
            UnaryOperator::Minus => Value::Number(-right.into_number(o.location)?),
            UnaryOperator::Not => Value::Boolean(!right.into_boolean(o.location)?),
        })
    }

    fn evaluate_binary(
        &self,
        left: Expression,
        operator: CodeBinaryOperator,
        right: Expression,
    ) -> RuntimeResult<Value> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;
        let loc = operator.location;
        Ok(match operator.op {
            // Math
            BinaryOperator::Subtract => {
                Value::Number(left.into_number(loc)? - right.into_number(loc)?)
            }
            BinaryOperator::Divide => {
                Value::Number(left.into_number(loc)? / right.into_number(loc)?)
            }
            BinaryOperator::Multiply => {
                Value::Number(left.into_number(loc)? * right.into_number(loc)?)
            }
            // Comparison
            BinaryOperator::Less => {
                Value::Boolean(left.into_number(loc)? < right.into_number(loc)?)
            }
            BinaryOperator::LessEquals => {
                Value::Boolean(left.into_number(loc)? <= right.into_number(loc)?)
            }
            BinaryOperator::Greater => {
                Value::Boolean(left.into_number(loc)? > right.into_number(loc)?)
            }
            BinaryOperator::GreaterEquals => {
                Value::Boolean(left.into_number(loc)? >= right.into_number(loc)?)
            }
            // Equality
            BinaryOperator::Equals => Value::Boolean(left == right),
            BinaryOperator::NotEquals => Value::Boolean(left != right),
            // Add
            BinaryOperator::Add => {
                if let Ok(left) = left.clone().into_number(loc) {
                    let right = right.into_number(loc)?;
                    Value::Number(left + right)
                } else if let Ok(left) = left.clone().into_string(loc) {
                    let right = right.into_string(loc)?;
                    Value::String(left + &right)
                } else {
                    return Err(RuntimeError::TypeErrorMultiple(
                        loc,
                        vec![Type::Number, Type::String],
                        left.value_type(),
                    ));
                }
            }
        })
    }
}
