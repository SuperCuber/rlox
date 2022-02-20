use std::{
    cell::RefCell,
    collections::{btree_map::Entry, BTreeMap},
    rc::Rc,
};

use crate::{error::RuntimeErrorKind, value::Value};

pub struct Environment {
    enclosing: Option<Rc<RefCell<Environment>>>,
    values: BTreeMap<String, Value>,
}

impl Environment {
    pub fn new() -> Rc<RefCell<Environment>> {
        Rc::new(RefCell::new(Environment {
            values: BTreeMap::new(),
            enclosing: None,
        }))
    }

    pub fn new_inside(enclosing: Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
        Rc::new(RefCell::new(Environment {
            values: BTreeMap::new(),
            enclosing: Some(enclosing),
        }))
    }

    pub fn pop(&self) -> Option<Rc<RefCell<Environment>>> {
        self.enclosing.clone()
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: String) -> Result<Value, RuntimeErrorKind> {
        match self.values.get(&name) {
            Some(v) => Ok(v.clone()),
            None => match &self.enclosing {
                Some(e) => e.borrow().get(name),
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
                Some(enc) => enc.borrow_mut().assign(name, value),
                None => Err(RuntimeErrorKind::UndefinedVariable(name)),
            },
        }
    }
}
