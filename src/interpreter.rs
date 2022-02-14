use crate::{
    error::RuntimeError,
    expression::{
        BinaryOperator, CodeBinaryOperator, CodeUnaryOperator, Expression, UnaryOperator,
    },
    token::Literal,
    value::Value,
};

type RuntimeResult<T> = Result<T, RuntimeError>;

pub fn interpret(e: Expression) -> Result<(), RuntimeError> {
    let value = evaluate(e)?;
    println!("{value}");
    Ok(())
}

fn evaluate(expression: Expression) -> RuntimeResult<Value> {
    match expression {
        Expression::Literal(l) => Ok(evaluate_literal(l)),
        Expression::Grouping(e) => evaluate_grouping(*e),
        Expression::Unary(o, r) => evaluate_unary(o, *r),
        Expression::Binary(l, o, r) => evaluate_binary(*l, o, *r),
    }
}

fn evaluate_literal(literal: Literal) -> Value {
    match literal {
        Literal::String(s) => Value::String(s),
        Literal::Number(n) => Value::Number(n),
        Literal::Boolean(b) => Value::Boolean(b),
        Literal::Nil => Value::Nil,
    }
}

fn evaluate_grouping(e: Expression) -> RuntimeResult<Value> {
    evaluate(e)
}

fn evaluate_unary(o: CodeUnaryOperator, r: Expression) -> RuntimeResult<Value> {
    let right = evaluate(r)?;

    Ok(match o.op {
        UnaryOperator::Minus => Value::Number(-right.into_number(o.location)?),
        UnaryOperator::Not => Value::Boolean(!right.into_boolean(o.location)?),
    })
}

fn evaluate_binary(
    left: Expression,
    operator: CodeBinaryOperator,
    right: Expression,
) -> RuntimeResult<Value> {
    let left = evaluate(left)?;
    let right = evaluate(right)?;
    let loc = operator.location;
    Ok(match operator.op {
        // TODO: handle errors
        // Math
        BinaryOperator::Subtract => Value::Number(left.into_number(loc)? - right.into_number(loc)?),
        BinaryOperator::Divide => Value::Number(left.into_number(loc)? / right.into_number(loc)?),
        BinaryOperator::Multiply => Value::Number(left.into_number(loc)? * right.into_number(loc)?),
        // Comparison
        BinaryOperator::Less => Value::Boolean(left.into_number(loc)? < right.into_number(loc)?),
        BinaryOperator::LessEquals => {
            Value::Boolean(left.into_number(loc)? <= right.into_number(loc)?)
        }
        BinaryOperator::Greater => Value::Boolean(left.into_number(loc)? > right.into_number(loc)?),
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
            } else if let Ok(left) = left.into_string(loc) {
                let right = right.into_string(loc)?;
                Value::String(left + &right)
            } else {
                panic!("type error");
            }
        }
    })
}
