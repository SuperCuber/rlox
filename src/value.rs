use std::fmt::Display;

use crate::error::RuntimeError;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Number(f32),
    Boolean(bool),
    Nil,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    String,
    Number,
    Boolean,
    Nil,
}

impl Value {
    pub fn into_string(self) -> Result<String, RuntimeError> {
        match self {
            Value::String(s) => Ok(s),
            v => Err(RuntimeError::TypeError(Type::String, v.value_type())),
        }
    }

    pub fn into_number(self) -> Result<f32, RuntimeError> {
        match self {
            Value::Number(s) => Ok(s),
            v => Err(RuntimeError::TypeError(Type::Number, v.value_type())),
        }
    }

    pub fn into_boolean(self) -> Result<bool, RuntimeError> {
        match self {
            Value::Boolean(s) => Ok(s),
            v => Err(RuntimeError::TypeError(Type::Boolean, v.value_type())),
        }
    }

    pub fn value_type(&self) -> Type {
        match self {
            Value::String(_) => Type::String,
            Value::Number(_) => Type::Number,
            Value::Boolean(_) => Type::Boolean,
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
            Value::String(s) => write!(f, "\"{s}\""),
            Value::Number(n) => {
                let n = *n;
                if n.floor() == n {
                    write!(f, "{}", n as i32)
                } else {
                    write!(f, "{n}")
                }
            }
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Nil => write!(f, "nil"),
        }
    }
}
