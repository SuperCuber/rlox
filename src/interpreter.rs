use std::time::UNIX_EPOCH;

use crate::{
    ast::{BinaryOperator, CodeExpression, Expression, Statement, UnaryOperator},
    environment::Environment,
    error::{RuntimeError, RuntimeErrorKind, WithLocation},
    token::Literal,
    value::{shared, LoxCallable, SharedValue, Type, Value},
};

pub type RuntimeResult<T> = Result<T, RuntimeError>;

pub struct Interpreter {
    pub environment: Environment,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let mut globals = Environment::new();
        globals.define(
            "clock".into(),
            shared(Value::Callable(LoxCallable::NativeFunction(
                "clock".into(),
                0,
                Box::new(clock),
            ))),
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
            Statement::Block(b) => self.execute_statement_block(b),
            Statement::If(condition, then_branch, else_branch) => {
                self.execute_if(condition, *then_branch, else_branch)
            }
            Statement::While(condition, body) => self.execute_while(condition, *body),
            Statement::Function(name, params, body) => self.execute_fun(name, params, body),
            Statement::Return(expr) => self.execute_return(expr),
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
            shared(Value::Nil)
        };
        self.environment.define(name, value);

        Ok(())
    }

    pub fn execute_statement_block(&mut self, statements: Vec<Statement>) -> RuntimeResult<()> {
        // Trickery to momentarily move the environment out
        let mut dummy_env = Environment::new();
        std::mem::swap(&mut self.environment, &mut dummy_env);
        self.environment = Environment::new_inside(dummy_env);

        let res = self.execute_block(statements);

        let mut dummy_env = Environment::new();
        std::mem::swap(&mut self.environment, &mut dummy_env);
        self.environment = dummy_env.pop().unwrap();
        res
    }

    fn execute_block(&mut self, statements: Vec<Statement>) -> RuntimeResult<()> {
        for statement in statements {
            self.execute(statement)?;
        }
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
        if condition.as_boolean().with_location(location)? {
            self.execute(then_branch)?;
        } else if let Some(else_branch) = else_branch {
            self.execute(*else_branch)?;
        }
        Ok(())
    }

    fn execute_while(&mut self, condition: CodeExpression, body: Statement) -> RuntimeResult<()> {
        while self
            .evaluate(condition.clone())?
            .as_boolean()
            .with_location(condition.location)?
        {
            self.execute(body.clone())?;
        }
        Ok(())
    }

    fn execute_fun(
        &mut self,
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
    ) -> RuntimeResult<()> {
        let function = shared(Value::Callable(LoxCallable::LoxFunction {
            name: name.clone(),
            params,
            body,
            closure: self.environment.clone(),
        }));
        self.environment.define(name, function);
        Ok(())
    }

    fn execute_return(&mut self, expression: Option<CodeExpression>) -> RuntimeResult<()> {
        let value = expression
            .map(|e| self.evaluate(e))
            .transpose()?
            .unwrap_or_else(|| shared(Value::Nil));
        Err(RuntimeError {
            location: (0, 0),
            value: RuntimeErrorKind::Returning(value),
        })
    }

    pub fn evaluate(&mut self, expression: CodeExpression) -> RuntimeResult<SharedValue> {
        let loc = expression.location;
        match expression.value {
            Expression::Literal(l) => Ok(shared(self.evaluate_literal(l))),
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
    ) -> RuntimeResult<SharedValue> {
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
    ) -> RuntimeResult<SharedValue> {
        let right = self.evaluate(r)?;

        Ok(match o {
            UnaryOperator::Minus => {
                shared(Value::Number(-right.as_number().with_location(location)?))
            }
            UnaryOperator::Not => {
                shared(Value::Boolean(!right.as_boolean().with_location(location)?))
            }
        })
    }

    fn evaluate_binary(
        &mut self,
        location: (usize, usize),
        left: CodeExpression,
        operator: BinaryOperator,
        right: CodeExpression,
    ) -> RuntimeResult<SharedValue> {
        let left = self.evaluate(left)?;
        let right = self.evaluate(right)?;

        // budget try-catch
        let res: Result<Value, RuntimeErrorKind> = (|| {
            let res = match operator {
                // Math
                BinaryOperator::Subtract => Value::Number(left.as_number()? - right.as_number()?),
                BinaryOperator::Divide => Value::Number(left.as_number()? / right.as_number()?),
                BinaryOperator::Multiply => Value::Number(left.as_number()? * right.as_number()?),
                // Comparison
                BinaryOperator::Less => Value::Boolean(left.as_number()? < right.as_number()?),
                BinaryOperator::LessEquals => {
                    Value::Boolean(left.as_number()? <= right.as_number()?)
                }
                BinaryOperator::Greater => Value::Boolean(left.as_number()? > right.as_number()?),
                BinaryOperator::GreaterEquals => {
                    Value::Boolean(left.as_number()? >= right.as_number()?)
                }
                // Equality
                BinaryOperator::Equals => Value::Boolean(left.eq(&right)),
                BinaryOperator::NotEquals => Value::Boolean(!left.eq(&right)),
                // Logical - short circuiting
                BinaryOperator::And => {
                    let left = left.as_boolean()?;
                    let right = right.as_boolean()?;
                    if !left {
                        Value::Boolean(false)
                    } else {
                        Value::Boolean(right)
                    }
                }
                BinaryOperator::Or => {
                    let left = left.as_boolean()?;
                    let right = right.as_boolean()?;
                    if left {
                        Value::Boolean(true)
                    } else {
                        Value::Boolean(right)
                    }
                }
                // Add
                BinaryOperator::Add => {
                    if let Ok(left) = left.as_number() {
                        let right = right.as_number()?;
                        Value::Number(left + right)
                    } else if let Ok(left) = left.as_string() {
                        let right = right.as_string()?;
                        Value::String(left.to_string() + right)
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
        res.map(shared).with_location(location)
    }

    fn evaluate_call(
        &mut self,
        location: (usize, usize),
        callee: CodeExpression,
        args_expressions: Vec<CodeExpression>,
    ) -> RuntimeResult<SharedValue> {
        let callee = self.evaluate(callee)?;
        let callee = callee;
        let callee = callee.as_callable().with_location(location)?;

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

fn clock(_interpreter: &mut Interpreter, _args: Vec<SharedValue>) -> RuntimeResult<SharedValue> {
    Ok(shared(Value::Number(
        std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f32(),
    )))
}
