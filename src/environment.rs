use std::collections::BTreeMap;

use crate::{
    error::{Located, RuntimeErrorKind},
    interpreter::RuntimeResult,
    value::Value,
};

pub struct Environment {
    values: BTreeMap<String, Value>,
}

impl Environment {
    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: Located<String>) -> RuntimeResult<Value> {
        self.values.get(&name.value).cloned().ok_or(Located {
            location: name.location,
            value: RuntimeErrorKind::UndefinedVariable(name.value),
        })
    }
}
