#![allow(unused)]

use crate::ast::{Expr, Statement};
use crate::error::{ErrorKind, Lox, report};
use crate::token::{self, Token, TokenType};
use crate::value::{self, *};

// pub fn interpret(expr: &Expr, lox: &mut Lox) -> Option<Value> {
//     evaluate(expr, lox)
// }

pub fn execute(stmt: &Statement, lox: &mut Lox) {
    match stmt {
        Statement::PrintStatement(expr) => {
            // println!("{:?}", expr);
            if let Some(value) = evaluate(expr, lox) {
                println!("{value}");
            }
        }
        Statement::ExprStatement(expr) => {
            evaluate(expr, lox);
        }
    }
}

pub fn interpret(statements: &Vec<Statement>, lox: &mut Lox) {
    for stmt in statements {
        execute(stmt, lox);
    }
}

fn evaluate(expression: &Expr, lox: &mut Lox) -> Option<Value> {
    match expression {
        Expr::Literal { value: literal } => Some(Value::from(literal.clone()?)),
        Expr::Grouping { expression: expr } => evaluate(expr, lox),
        Expr::Unary {
            operator: op,
            expression: expr,
        } => evaluate_unary(expr, op, lox),
        Expr::Binary {
            left,
            operator: op,
            right,
        } => evaluate_binary(left, right, op, lox),
    }
}

fn evaluate_unary(expression: &Expr, operator: &Token, lox: &mut Lox) -> Option<Value> {
    let right = evaluate(expression, lox)?;
    match operator.token_type {
        TokenType::Minus => match right {
            Value::Num(num) => Some(Value::Num(-num)),
            _ => {
                let error = ErrorKind::WithLocation {
                    message: String::from("Invalid operand for unary minus"),
                    line: u32::try_from(operator.line).unwrap(),
                    col: None,
                };
                report(lox, error);
                None
            }
        },
        TokenType::Bang => Some(Value::Bool(is_truthy(&right))),
        _ => unreachable!(
            "parser produced non unary operator {:?} in an unary expression",
            operator.token_type
        ),
    }
}

fn evaluate_binary(left: &Expr, right: &Expr, operator: &Token, lox: &mut Lox) -> Option<Value> {
    let left = evaluate(left, lox)?;
    let right = evaluate(right, lox)?;

    match operator.token_type {
        TokenType::Minus => match (left, right) {
            (Value::Num(a), Value::Num(b)) => Some(Value::Num(a - b)),
            (_, _) => {
                let error = ErrorKind::WithLocation {
                    message: String::from("Types dont suit subtraction"),
                    line: u32::try_from(operator.line).unwrap(),
                    col: None,
                };
                report(lox, error);
                None
            }
        },

        TokenType::Plus => match (left, right) {
            (Value::Num(a), Value::Num(b)) => Some(Value::Num(a + b)),
            (Value::Str(a), Value::Str(b)) => Some(Value::Str(a + &b)),
            (_, _) => {
                let error = ErrorKind::WithLocation {
                    message: String::from("Types dont suit addition"),
                    line: u32::try_from(operator.line).unwrap(),
                    col: None,
                };
                report(lox, error);
                None
            }
        },

        TokenType::Star => match (left, right) {
            (Value::Num(a), Value::Num(b)) => Some(Value::Num(a * b)),
            (Value::Str(a), Value::Num(b)) | (Value::Num(b), Value::Str(a)) => {
                if (b < 0.0 || b.fract() != 0.0) {
                    let error = ErrorKind::WithLocation {
                        message: format!("Can't multiply a string with {b}"),
                        line: u32::try_from(operator.line).unwrap(),
                        col: None,
                    };
                    report(lox, error);
                    return None;
                }
                Some(Value::Str(a.repeat(b as usize)))
            }
            (_, _) => {
                let error = ErrorKind::WithLocation {
                    message: String::from("Types dont suit multiplication"),
                    line: u32::try_from(operator.line).unwrap(),
                    col: None,
                };
                report(lox, error);
                None
            }
        },

        TokenType::Slash => match (left, right) {
            (Value::Num(a), Value::Num(b)) => {
                if (b == 0.0) {
                    let error = ErrorKind::WithLocation {
                        message: String::from("Attempted division by zero"),
                        line: u32::try_from(operator.line).unwrap(),
                        col: None,
                    };
                    report(lox, error);
                    return None;
                }
                Some(Value::Num(a / b))
            }

            (_, _) => {
                let error = ErrorKind::WithLocation {
                    message: String::from("Types dont suit division"),
                    line: u32::try_from(operator.line).unwrap(),
                    col: None,
                };
                report(lox, error);
                None
            }
        },

        TokenType::Greater => match (left, right) {
            (Value::Num(a), Value::Num(b)) => Some(Value::Bool(a > b)),
            (Value::Str(a), Value::Str(b)) => Some(Value::Bool(a > b)),
            (_, _) => {
                let error = ErrorKind::WithLocation {
                    message: String::from("Types cant be compared"),
                    line: u32::try_from(operator.line).unwrap(),
                    col: None,
                };
                report(lox, error);
                None
            }
        },

        TokenType::GreaterEqual => match (left, right) {
            (Value::Num(a), Value::Num(b)) => Some(Value::Bool(a >= b)),
            (Value::Str(a), Value::Str(b)) => Some(Value::Bool(a >= b)),
            (_, _) => {
                let error = ErrorKind::WithLocation {
                    message: String::from("Types cant be compared"),
                    line: u32::try_from(operator.line).unwrap(),
                    col: None,
                };
                report(lox, error);
                None
            }
        },

        TokenType::Less => match (left, right) {
            (Value::Num(a), Value::Num(b)) => Some(Value::Bool(a < b)),
            (Value::Str(a), Value::Str(b)) => Some(Value::Bool(a < b)),
            (_, _) => {
                let error = ErrorKind::WithLocation {
                    message: String::from("Types cant be compared"),
                    line: u32::try_from(operator.line).unwrap(),
                    col: None,
                };
                report(lox, error);
                None
            }
        },

        TokenType::LessEqual => match (left, right) {
            (Value::Num(a), Value::Num(b)) => Some(Value::Bool(a <= b)),
            (Value::Str(a), Value::Str(b)) => Some(Value::Bool(a <= b)),
            (_, _) => {
                let error = ErrorKind::WithLocation {
                    message: String::from("Types cant be compared"),
                    line: u32::try_from(operator.line).unwrap(),
                    col: None,
                };
                report(lox, error);
                None
            }
        },

        TokenType::EqualEqual => match (left, right) {
            (Value::Num(a), Value::Num(b)) => Some(Value::Bool(a == b)),
            (Value::Str(a), Value::Str(b)) => Some(Value::Bool(a == b)),
            (Value::Nil, Value::Nil) => Some(Value::Bool(true)),
            (Value::Nil, _) => Some(Value::Bool(false)),
            (_, _) => {
                let error = ErrorKind::WithLocation {
                    message: String::from("Types cant be compared"),
                    line: u32::try_from(operator.line).unwrap(),
                    col: None,
                };
                report(lox, error);
                None
            }
        },

        TokenType::BangEqual => match (left, right) {
            (Value::Num(a), Value::Num(b)) => Some(Value::Bool(a != b)),
            (Value::Str(a), Value::Str(b)) => Some(Value::Bool(a != b)),
            (Value::Nil, Value::Nil) => Some(Value::Bool(false)),
            (Value::Nil, _) => Some(Value::Bool(true)),
            (_, _) => {
                let error = ErrorKind::WithLocation {
                    message: String::from("Types cant be compared"),
                    line: u32::try_from(operator.line).unwrap(),
                    col: None,
                };
                report(lox, error);
                None
            }
        },

        _ => None,
    }
}

//helpers
fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Str(str) => !(str.is_empty()),
        Value::Num(num) => *num != f64::from(0),
        Value::Bool(bool) => *bool,
        Value::Nil => false,
    }
}
