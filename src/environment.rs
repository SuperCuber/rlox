use std::collections::BTreeMap;

use crate::{error::RuntimeErrorKind, value::Value};

pub struct Environment {
    values: BTreeMap<String, Value>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            values: BTreeMap::new(),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: String) -> Result<Value, RuntimeErrorKind> {
        self.values
            .get(&name)
            .cloned()
            .ok_or(RuntimeErrorKind::UndefinedVariable(name))
    }

    pub fn assign(&mut self, name: String, value: Value) -> Result<(), RuntimeErrorKind> {
        #[allow(clippy::map_entry)] // ew
        if self.values.contains_key(&name) {
            self.values.insert(name, value);
            Ok(())
        } else {
            Err(RuntimeErrorKind::UndefinedVariable(name))
        }
    }
}
