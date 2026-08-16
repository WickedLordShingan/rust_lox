#![allow(unused)]
use crate::value::Value;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub type Env = Rc<RefCell<Environment>>;

#[derive(Debug)]
pub struct Environment {
    pub values: HashMap<String, Value>,
    pub parent: Option<Env>,
}

impl Environment {
    pub fn init() -> Env {
        Rc::new(RefCell::new(Environment {
            values: HashMap::new(),
            parent: None,
        }))
    }

    // create a new child scope hanging off `parent`
    pub fn child(parent: &Env) -> Env {
        Rc::new(RefCell::new(Environment {
            values: HashMap::new(),
            parent: Some(parent.clone()),
        }))
    }

    pub fn define(&mut self, name: &str, val: Value) {
        self.values.insert(name.to_string(), val);
    }

    pub fn assign(env: &Env, name: &str, val: Value) -> Option<()> {
        if env.borrow().values.contains_key(name) {
            env.borrow_mut()
                .values
                .insert(name.to_string(), val.clone());
            return Some(());
        }
        let parent = env.borrow().parent.clone();
        match parent {
            Some(p) => Environment::assign(&p, name, val),
            //then at every call to assign you need to make sure that error handling is done
            //properly
            None => None,
        }
    }

    pub fn lookup(env: &Env, name: &str) -> Option<Value> {
        if let Some(val) = env.borrow().values.get(name) {
            return Some(val.clone());
        }
        let parent = env.borrow().parent.clone()?;
        Environment::lookup(&parent, name)
    }
}
