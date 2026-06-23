#![allow(unused)]
use crate::value::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Environment {
    values: HashMap<String, Value>,
    parent: Option<Box<Environment>>,
}

impl Environment {
    pub fn init() -> Self {
        Self {
            values: HashMap::new(),
            parent: None,
        }
    }

    pub fn define(&mut self, name: &str, val: Value) {
        self.values.insert(name.to_string(), val);
    }

    pub fn assign(&mut self, name: &str, val: Value) -> Result<(), String> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), val);
            return Ok(());
        }
        if let Some(parent) = &mut self.parent {
            return parent.assign(name, val);
        }
        Err(format!("Undefined variable name {name}"))
    }

    pub fn lookup(&self, name: &str) -> Option<&Value> {
        if let Some(val) = self.values.get(name) {
            return Some(val);
        }
        self.parent.as_ref()?.lookup(name)
    }

    pub fn start_scope(&mut self) {
        self.parent = Some(Box::new(self.clone()));
        self.values = HashMap::new();
    }

    pub fn end_scope(&mut self) {
        if let Some(parent) = self.parent.take() {
            let environ = *parent;
            self.values = environ.values;
            self.parent = environ.parent;
        }
    }
}
