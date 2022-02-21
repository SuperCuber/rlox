use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use crate::{ast::ResolvedVariable, error::RuntimeErrorKind, value::Value};

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

    pub fn get(&self, variable: ResolvedVariable) -> Result<Value, RuntimeErrorKind> {
        match variable.hops {
            Some(0) => self.values.get(&variable.name).cloned(),
            Some(h) => self
                .enclosing
                .as_ref()
                .expect("correctly calculated hops")
                .borrow()
                .get(ResolvedVariable {
                    hops: Some(h - 1),
                    name: variable.name.clone(),
                })
                .ok(),
            None => {
                if let Some(e) = &self.enclosing {
                    e.borrow().get(variable.clone()).ok()
                } else {
                    self.values.get(&variable.name).cloned()
                }
            }
        }
        .ok_or(RuntimeErrorKind::UndefinedVariable(variable.name))
    }

    pub fn assign(
        &mut self,
        variable: ResolvedVariable,
        value: Value,
    ) -> Result<(), RuntimeErrorKind> {
        match variable.hops {
            Some(0) => {
                self.values.insert(variable.name, value);
            }
            Some(h) => {
                self.enclosing
                    .as_ref()
                    .expect("correctly calculated hops")
                    .borrow_mut()
                    .assign(
                        ResolvedVariable {
                            hops: Some(h - 1),
                            name: variable.name,
                        },
                        value,
                    )?;
            }
            None => {
                if let Some(e) = &self.enclosing {
                    e.borrow_mut().assign(variable, value)?;
                } else {
                    self.values.insert(variable.name, value);
                }
            }
        }
        Ok(())
    }
}
