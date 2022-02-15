use std::collections::{btree_map::Entry, BTreeMap};

use crate::{error::RuntimeErrorKind, value::Value};

pub struct Environment {
    enclosing: Option<Box<Environment>>,
    values: BTreeMap<String, Value>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            values: BTreeMap::new(),
            enclosing: None,
        }
    }

    pub fn push_env(&mut self) {
        let mut enclosing = Environment::new();
        std::mem::swap(self, &mut enclosing);
        // Old environment is in `enclosing`, new environment is self
        self.enclosing = Some(Box::new(enclosing));
    }

    pub fn pop_env(&mut self) {
        let mut enclosing = self.enclosing.take().unwrap();
        std::mem::swap(self, &mut enclosing);
        // Enclosing now is self, self in the enclosing variable and will be dropped
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: String) -> Result<Value, RuntimeErrorKind> {
        match self.values.get(&name) {
            Some(v) => Ok(v.clone()),
            None => match &self.enclosing {
                Some(e) => e.get(name),
                None => Err(RuntimeErrorKind::UndefinedVariable(name)),
            },
        }
    }

    pub fn assign(&mut self, name: String, value: Value) -> Result<(), RuntimeErrorKind> {
        match self.values.entry(name.clone()) {
            Entry::Occupied(mut e) => {
                e.insert(value);
                Ok(())
            }
            Entry::Vacant(_) => match &mut self.enclosing {
                Some(enc) => enc.assign(name, value),
                None => Err(RuntimeErrorKind::UndefinedVariable(name)),
            },
        }
    }
}
