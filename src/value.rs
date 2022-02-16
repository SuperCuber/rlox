use std::{fmt::Display, rc::Rc};

use crate::{
    error::RuntimeErrorKind,
    interpreter::{Interpreter, RuntimeResult},
};

#[derive(Clone, PartialEq)]
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
            Value::Callable(LoxCallable::NativeFunction(name, _, _)) => {
                write!(f, "<native function {name}>")
            }
            Value::Callable(LoxCallable::LoxFunction(_)) => {
                write!(f, "<function>")
            }
            Value::Nil => write!(f, "nil"),
        }
    }
}

type Function = Rc<Box<dyn Fn(&mut Interpreter, Vec<Value>) -> RuntimeResult<Value>>>;

#[derive(Clone)]
pub enum LoxCallable {
    LoxFunction(Function),
    NativeFunction(String, usize, Function),
}

impl PartialEq for LoxCallable {
    fn eq(&self, other: &LoxCallable) -> bool {
        match (self, other) {
            (LoxCallable::LoxFunction(f1), LoxCallable::LoxFunction(f2)) => Rc::ptr_eq(f1, f2),
            (LoxCallable::NativeFunction(_, _, f1), LoxCallable::NativeFunction(_, _, f2)) => {
                Rc::ptr_eq(f1, f2)
            }
            _ => false,
        }
    }
}

impl LoxCallable {
    pub fn call(
        &mut self,
        interpreter: &mut Interpreter,
        args: Vec<Value>,
    ) -> RuntimeResult<Value> {
        match self {
            LoxCallable::LoxFunction(fun) => fun(interpreter, args),
            LoxCallable::NativeFunction(_, _, fun) => fun(interpreter, args),
        }
    }

    pub fn arity(&self) -> usize {
        todo!()
    }
}
