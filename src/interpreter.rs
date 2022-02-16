use std::rc::Rc;

use crate::{
    ast::{BinaryOperator, CodeExpression, Expression, Statement, UnaryOperator},
    environment::Environment,
    error::{RuntimeError, RuntimeErrorKind, WithLocation},
    token::Literal,
    value::{LoxCallable, Type, Value},
};

pub type RuntimeResult<T> = Result<T, RuntimeError>;

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let mut globals = Environment::new();
        globals.define(
            "clock".into(),
            Value::Callable(LoxCallable::NativeFunction(
                "clock".into(),
                0,
                Rc::new(Box::new(clock)),
            )),
        );
        Interpreter {
            environment: globals,
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
        value: Option<CodeExpression>,
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
        condition: CodeExpression,
        then_branch: Statement,
        else_branch: Option<Box<Statement>>,
    ) -> RuntimeResult<()> {
        let location = condition.location;
        let condition = self.evaluate(condition)?;
        if condition.into_boolean().with_location(location)? {
            self.execute(then_branch)?;
        } else if let Some(else_branch) = else_branch {
            self.execute(*else_branch)?;
        }
        Ok(())
    }

    fn execute_while(&mut self, condition: CodeExpression, body: Statement) -> RuntimeResult<()> {
        while self
            .evaluate(condition.clone())?
            .into_boolean()
            .with_location(condition.location)?
        {
            self.execute(body.clone())?;
        }
        Ok(())
    }

    pub fn evaluate(&mut self, expression: CodeExpression) -> RuntimeResult<Value> {
        let loc = expression.location;
        match expression.value {
            Expression::Literal(l) => Ok(self.evaluate_literal(l)),
            Expression::Assign(v, e) => self.evaluate_assign(loc, v, *e),
            Expression::Grouping(e) => self.evaluate(*e),
            Expression::Unary(o, r) => self.evaluate_unary(loc, o, *r),
            Expression::Binary(l, o, r) => self.evaluate_binary(loc, *l, o, *r),
            Expression::Variable(v) => self.environment.get(v).with_location(loc),
            Expression::Call(c, a) => self.evaluate_call(loc, *c, a),
        }
    }

    fn evaluate_assign(
        &mut self,
        location: (usize, usize),
        variable: String,
        expression: CodeExpression,
    ) -> RuntimeResult<Value> {
        let value = self.evaluate(expression)?;
        self.environment
            .assign(variable, value.clone())
            .with_location(location)?;
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

    fn evaluate_unary(
        &mut self,
        location: (usize, usize),
        o: UnaryOperator,
        r: CodeExpression,
    ) -> RuntimeResult<Value> {
        let right = self.evaluate(r)?;

        Ok(match o {
            UnaryOperator::Minus => Value::Number(-right.into_number().with_location(location)?),
            UnaryOperator::Not => Value::Boolean(!right.into_boolean().with_location(location)?),
        })
    }

    fn evaluate_binary(
        &mut self,
        location: (usize, usize),
        left: CodeExpression,
        operator: BinaryOperator,
        right: CodeExpression,
    ) -> RuntimeResult<Value> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        // budget try-catch
        let res: Result<Value, RuntimeErrorKind> = (|| {
            let res = match operator {
                // Math
                BinaryOperator::Subtract => {
                    Value::Number(left.into_number()? - right.into_number()?)
                }
                BinaryOperator::Divide => Value::Number(left.into_number()? / right.into_number()?),
                BinaryOperator::Multiply => {
                    Value::Number(left.into_number()? * right.into_number()?)
                }
                // Comparison
                BinaryOperator::Less => Value::Boolean(left.into_number()? < right.into_number()?),
                BinaryOperator::LessEquals => {
                    Value::Boolean(left.into_number()? <= right.into_number()?)
                }
                BinaryOperator::Greater => {
                    Value::Boolean(left.into_number()? > right.into_number()?)
                }
                BinaryOperator::GreaterEquals => {
                    Value::Boolean(left.into_number()? >= right.into_number()?)
                }
                // Equality
                BinaryOperator::Equals => Value::Boolean(left == right),
                BinaryOperator::NotEquals => Value::Boolean(left != right),
                // Logical - short circuiting
                BinaryOperator::And => {
                    let left = left.into_boolean()?;
                    let right = right.into_boolean()?;
                    if !left {
                        Value::Boolean(false)
                    } else {
                        Value::Boolean(right)
                    }
                }
                BinaryOperator::Or => {
                    let left = left.into_boolean()?;
                    let right = right.into_boolean()?;
                    if left {
                        Value::Boolean(true)
                    } else {
                        Value::Boolean(right)
                    }
                }
                // Add
                BinaryOperator::Add => {
                    if let Ok(left) = left.clone().into_number() {
                        let right = right.into_number()?;
                        Value::Number(left + right)
                    } else if let Ok(left) = left.clone().into_string() {
                        let right = right.into_string()?;
                        Value::String(left + &right)
                    } else {
                        return Err(RuntimeErrorKind::TypeErrorMultiple(
                            vec![Type::Number, Type::String],
                            left.value_type(),
                        ));
                    }
                }
            };
            Ok(res)
        })();
        res.with_location(location)
    }

    fn evaluate_call(
        &mut self,
        location: (usize, usize),
        callee: CodeExpression,
        args_expressions: Vec<CodeExpression>,
    ) -> RuntimeResult<Value> {
        let mut callee = self
            .evaluate(callee)?
            .into_callable()
            .with_location(location)?;

        let mut args = Vec::new();
        for arg in args_expressions {
            args.push(self.evaluate(arg)?);
        }

        if args.len() != callee.arity() {
            return Err(RuntimeError {
                location,
                value: RuntimeErrorKind::WrongArgsNum(args.len(), callee.arity()),
            });
        }

        callee.call(self, args)
    }
}

fn clock(_interpreter: &mut Interpreter, _args: Vec<Value>) -> RuntimeResult<Value> {
    todo!()
}
