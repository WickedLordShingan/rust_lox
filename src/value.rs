use std::{
    cell::RefCell,
    fmt::{self, Display, write},
    rc::Rc,
};

use crate::{
    ast::Statement,
    environment::Environment,
    token::{Literal, Token},
};

#[derive(Clone)]
pub enum Value {
    Str(String),
    Num(f64),
    Bool(bool),
    Nil,
    Function {
        name: String,
        params: Vec<Token>,
        body: Rc<Vec<Statement>>,
        closure: Rc<RefCell<Environment>>,
    },
    Foreign {
        arity: usize,
        name: String,
        implementation: Rc<dyn Fn(&[Value]) -> Result<Value, String>>,
    },
}

impl From<Literal> for Value {
    fn from(literal: Literal) -> Self {
        match literal {
            Literal::Str(string) => Self::Str(string),
            Literal::Num(number) => Self::Num(number),
            Literal::Bool(boolean) => Self::Bool(boolean),
            _ => Value::Nil,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Str(str) => write!(f, "String : {str}"),
            Self::Num(num) => {
                if num.fract() == 0.0 {
                    return write!(f, "Integer : {}", *num as i64);
                }
                write!(f, "Float : {num}")
            }
            Self::Bool(bool) => write!(f, "Boolean : {bool}"),
            Self::Nil => write!(f, "nandemonai desuga"),
            Self::Function { name, .. } => write!(f, "<fn> {name}"),
            Self::Foreign {
                arity,
                name,
                implementation: _,
            } => {
                return write!(f, "arity : {arity} name : {name}");
            }
        }
    }
}
