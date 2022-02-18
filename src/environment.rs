use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use crate::{error::RuntimeErrorKind, value::SharedValue};

// Clone is half-deep: variable contents are actually shared, but the mapping is cloned
#[derive(Debug, Clone)]
pub struct Environment {
    enclosing: Option<Box<Environment>>,
    values: BTreeMap<String, Rc<RefCell<SharedValue>>>,
}

impl Environment {
    pub fn new() -> Environment {
        Environment {
            values: BTreeMap::new(),
            enclosing: None,
        }
    }

    pub fn new_inside(enclosing: Environment) -> Environment {
        Environment {
            values: BTreeMap::new(),
            enclosing: Some(Box::new(enclosing)),
        }
    }

    pub fn pop(self) -> Option<Environment> {
        self.enclosing.map(|e| *e)
    }

    pub fn define(&mut self, name: String, value: SharedValue) {
        self.values.insert(name, Rc::new(RefCell::new(value)));
    }

    pub fn get(&self, name: String) -> Result<SharedValue, RuntimeErrorKind> {
        match self.values.get(&name) {
            Some(v) => Ok(v.borrow().clone()),
            None => match &self.enclosing {
                Some(e) => e.get(name),
                None => Err(RuntimeErrorKind::UndefinedVariable(name)),
            },
        }
    }

    pub fn assign(&self, name: String, value: SharedValue) -> Result<(), RuntimeErrorKind> {
        match self.values.get(&name) {
            Some(v) => {
                *v.borrow_mut() = value;
                Ok(())
            }
            None => match &self.enclosing {
                Some(enc) => enc.assign(name, value),
                None => Err(RuntimeErrorKind::UndefinedVariable(name)),
            },
        }
    }
}
