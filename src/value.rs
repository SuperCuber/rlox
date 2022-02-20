use std::{fmt::Debug, fmt::Display, rc::Rc};

use crate::{
    ast::Statement,
    error::{RuntimeError, RuntimeErrorKind},
    interpreter::{Interpreter, RuntimeResult},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Number(f32),
    Boolean(bool),
    Callable(LoxCallable),
    Nil,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    String,
    Number,
    Boolean,
    Callable,
    Nil,
}

impl Value {
    pub fn into_string(self) -> Result<String, RuntimeErrorKind> {
        match self {
            Value::String(s) => Ok(s),
            v => Err(RuntimeErrorKind::TypeError(Type::String, v.value_type())),
        }
    }

    pub fn into_number(self) -> Result<f32, RuntimeErrorKind> {
        match self {
            Value::Number(s) => Ok(s),
            v => Err(RuntimeErrorKind::TypeError(Type::Number, v.value_type())),
        }
    }

    pub fn into_boolean(self) -> Result<bool, RuntimeErrorKind> {
        match self {
            Value::Boolean(s) => Ok(s),
            v => Err(RuntimeErrorKind::TypeError(Type::Boolean, v.value_type())),
        }
    }

    pub fn into_callable(self) -> Result<LoxCallable, RuntimeErrorKind> {
        match self {
            Value::Callable(s) => Ok(s),
            v => Err(RuntimeErrorKind::TypeError(Type::Callable, v.value_type())),
        }
    }

    pub fn value_type(&self) -> Type {
        match self {
            Value::String(_) => Type::String,
            Value::Number(_) => Type::Number,
            Value::Boolean(_) => Type::Boolean,
            Value::Callable(_) => Type::Callable,
            Value::Nil => Type::Nil,
        }
    }

    // pub fn into_nil(self) -> Option<()> {
    //     match self {
    //         Value::Nil => Some(()),
    //         _ => None,
    //     }
    // }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "{s}"),
            Value::Number(n) => {
                let n = *n;
                if n.is_nan() {
                    write!(f, "NaN")
                } else if n.is_infinite() && n.is_sign_positive() {
                    write!(f, "Inf")
                } else if n.is_infinite() && n.is_sign_negative() {
                    write!(f, "-Inf")
                } else if n.floor() == n {
                    write!(f, "{}", n as i32)
                } else {
                    write!(f, "{n}")
                }
            }
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Callable(LoxCallable::NativeFunction(name, ..)) => {
                write!(f, "<native function {name}>")
            }
            Value::Callable(LoxCallable::LoxFunction { name, .. }) => {
                write!(f, "<function {name}>")
            }
            Value::Nil => write!(f, "nil"),
        }
    }
}

type Function = Rc<Box<dyn Fn(&mut Interpreter, Vec<Value>) -> RuntimeResult<Value>>>;

#[derive(Clone)]
pub enum LoxCallable {
    LoxFunction {
        name: String,
        params: Vec<String>,
        body: Vec<Statement>,
    },
    NativeFunction(String, usize, Function),
}

impl Debug for LoxCallable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LoxFunction { name, params, body } => f
                .debug_struct("LoxFunction")
                .field("name", name)
                .field("params", params)
                .field("body", body)
                .finish(),
            Self::NativeFunction(arg0, arg1, _) => f
                .debug_tuple("NativeFunction")
                .field(arg0)
                .field(arg1)
                .field(&"<native code>")
                .finish(),
        }
    }
}

impl PartialEq for LoxCallable {
    fn eq(&self, other: &LoxCallable) -> bool {
        match (self, other) {
            (
                LoxCallable::LoxFunction { name: name1, .. },
                LoxCallable::LoxFunction { name: name2, .. },
            ) => name1 == name2,
            (LoxCallable::NativeFunction(_, _, f1), LoxCallable::NativeFunction(_, _, f2)) => {
                Rc::ptr_eq(f1, f2)
            }
            _ => false,
        }
    }
}

impl LoxCallable {
    pub fn call(self, interpreter: &mut Interpreter, args: Vec<Value>) -> RuntimeResult<Value> {
        match self {
            LoxCallable::LoxFunction { params, body, .. } => {
                interpreter.environment.push_env();
                for (param, arg) in params.into_iter().zip(args.into_iter()) {
                    interpreter.environment.define(param, arg)
                }
                let ans = interpreter.execute_block(body);
                interpreter.environment.pop_env();
                match ans {
                    Ok(()) => Ok(Value::Nil),
                    Err(RuntimeError {
                        value: RuntimeErrorKind::Returning(v),
                        ..
                    }) => Ok(v),
                    Err(e) => Err(e),
                }
            }
            LoxCallable::NativeFunction(_, _, fun) => fun(interpreter, args),
        }
    }

    pub fn arity(&self) -> usize {
        match self {
            LoxCallable::LoxFunction { params, .. } => params.len(),
            LoxCallable::NativeFunction(_, a, _) => *a,
        }
    }
}
