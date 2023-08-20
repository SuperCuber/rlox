use std::{
    cell::RefCell,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    ast::{
        BinaryOperator, Expression, ResolvedCodeExpression, ResolvedStatement, ResolvedVariable,
        UnaryOperator,
    },
    environment::Environment,
    error::{RuntimeError, RuntimeErrorKind, WithLocation},
    token::Literal,
    value::{LoxCallable, Type, Value},
};

pub type RuntimeResult<T> = Result<T, RuntimeError>;

pub struct Interpreter {
    pub environment: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        let globals = Environment::new();
        globals.borrow_mut().define(
            "debug".into(),
            Value::Callable(LoxCallable::NativeFunction(
                "debug".into(),
                1,
                Rc::new(Box::new(debug)),
            )),
        );
        globals.borrow_mut().define(
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

    pub fn interpret(&mut self, program: Vec<ResolvedStatement>) -> RuntimeResult<()> {
        for statement in program {
            self.execute(statement)?;
        }
        Ok(())
    }

    fn execute(&mut self, statement: ResolvedStatement) -> RuntimeResult<()> {
        match statement {
            ResolvedStatement::Expression(expr) => self.evaluate(expr).map(|_| ()),
            ResolvedStatement::Print(expr) => {
                println!("{}", self.evaluate(expr)?);
                Ok(())
            }
            ResolvedStatement::Var(name, value) => self.execute_statement_var(name, value),
            ResolvedStatement::Block(b) => self.execute_block_statement(b),
            ResolvedStatement::If(condition, then_branch, else_branch) => {
                self.execute_if(condition, *then_branch, else_branch)
            }
            ResolvedStatement::While(condition, body) => self.execute_while(condition, *body),
            ResolvedStatement::Function(name, params, body) => self.execute_fun(name, params, body),
            ResolvedStatement::Return(expr) => self.execute_return(expr),
        }
    }

    fn execute_statement_var(
        &mut self,
        name: String,
        value: Option<ResolvedCodeExpression>,
    ) -> Result<(), RuntimeError> {
        let value = if let Some(e) = value {
            self.evaluate(e)?
        } else {
            Value::Nil
        };
        self.environment.borrow_mut().define(name, value);

        Ok(())
    }

    pub fn execute_block_statement(
        &mut self,
        statements: Vec<ResolvedStatement>,
    ) -> RuntimeResult<()> {
        self.environment = Environment::new_inside(self.environment.clone());

        let res = self.execute_block(statements);

        self.environment = self.environment.clone().borrow().pop().unwrap();
        res
    }

    fn execute_block(&mut self, statements: Vec<ResolvedStatement>) -> RuntimeResult<()> {
        for statement in statements {
            self.execute(statement)?;
        }
        Ok(())
    }

    fn execute_if(
        &mut self,
        condition: ResolvedCodeExpression,
        then_branch: ResolvedStatement,
        else_branch: Option<Box<ResolvedStatement>>,
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

    fn execute_while(
        &mut self,
        condition: ResolvedCodeExpression,
        body: ResolvedStatement,
    ) -> RuntimeResult<()> {
        while self
            .evaluate(condition.clone())?
            .into_boolean()
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
        body: Vec<ResolvedStatement>,
    ) -> RuntimeResult<()> {
        let function = Value::Callable(LoxCallable::LoxFunction {
            name: name.clone(),
            params,
            body,
            closure: self.environment.clone(),
        });
        self.environment.borrow_mut().define(name, function);
        Ok(())
    }

    fn execute_return(&mut self, expression: Option<ResolvedCodeExpression>) -> RuntimeResult<()> {
        let value = expression
            .map(|e| self.evaluate(e))
            .transpose()?
            .unwrap_or(Value::Nil);
        Err(RuntimeError {
            location: (0, 0),
            value: RuntimeErrorKind::Returning(value),
        })
    }

    pub fn evaluate(&mut self, expression: ResolvedCodeExpression) -> RuntimeResult<Value> {
        let loc = expression.location;
        match expression.value {
            Expression::Literal(l) => Ok(self.evaluate_literal(l)),
            Expression::Assign(v, e) => self.evaluate_assign(loc, v, *e),
            Expression::Grouping(e) => self.evaluate(*e),
            Expression::Unary(o, r) => self.evaluate_unary(loc, o, *r),
            Expression::Binary(l, o, r) => self.evaluate_binary(loc, *l, o, *r),
            Expression::Variable(v) => self.environment.borrow().get(v).with_location(loc),
            Expression::Call(c, a) => self.evaluate_call(loc, *c, a),
        }
    }

    fn evaluate_assign(
        &mut self,
        location: (usize, usize),
        variable: ResolvedVariable,
        expression: ResolvedCodeExpression,
    ) -> RuntimeResult<Value> {
        let value = self.evaluate(expression)?;
        self.environment
            .borrow_mut()
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
        r: ResolvedCodeExpression,
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
        left: ResolvedCodeExpression,
        operator: BinaryOperator,
        right: ResolvedCodeExpression,
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
        callee: ResolvedCodeExpression,
        args_expressions: Vec<ResolvedCodeExpression>,
    ) -> RuntimeResult<Value> {
        let callee = self
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

        callee.call(self, args, location)
    }
}

fn clock(_interpreter: &mut Interpreter, _args: Vec<Value>) -> Result<Value, RuntimeErrorKind> {
    Ok(Value::Number(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64(),
    ))
}

fn debug(_interpreter: &mut Interpreter, args: Vec<Value>) -> Result<Value, RuntimeErrorKind> {
    let &[value] = &args.as_slice() else {
        return Err(RuntimeErrorKind::WrongArgsNum(args.len(), 1));
    };

    Ok(Value::String(format!("{:?}", value)))
}
