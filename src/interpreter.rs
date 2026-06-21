#![allow(unused)]

use crate::ast::{Expr, Statement};
use crate::environment::{self, Environment};
use crate::error::{ErrorKind, Lox, report};
use crate::token::{self, Token, TokenType};
use crate::value::{self, *};

pub fn interpret(statements: &Vec<Statement>, lox: &mut Lox, environment: &mut Environment) {
    for stmt in statements {
        execute(stmt, lox, environment);
    }
}

pub fn execute(stmt: &Statement, lox: &mut Lox, env: &mut Environment) {
    match stmt {
        Statement::PrintStatement(expr) => {
            // println!("{:?}", expr);
            if let Some(value) = evaluate(expr, lox, env) {
                println!("{value}");
            }
        }
        Statement::ExprStatement(expr) => {
            evaluate(expr, lox, env);
        }
        Statement::AssignStatement { token, expression } => {
            var_declaration(token, expression, lox, env);
        }
        Statement::BlockStatement(statements) => {
            env.start_scope();
            for statement in statements {
                execute(statement, lox, env);
                if lox.had_error {
                    break;
                }
            }
            env.end_scope();
        }
    }
}

fn var_declaration(
    token: &Option<Token>,
    expression: &Option<Expr>,
    lox: &mut Lox,
    env: &mut Environment,
) {
    let value = match expression {
        Some(expr) => evaluate(expr, lox, env).unwrap_or(Value::Nil),
        None => Value::Nil,
    };
    if let Some(tok) = token {
        env.define(&tok.lexeme, value);
    }
}

fn evaluate(expression: &Expr, lox: &mut Lox, env: &mut Environment) -> Option<Value> {
    match expression {
        Expr::Literal { value: literal } => Some(Value::from(literal.clone()?)),
        Expr::Grouping { expression: expr } => evaluate(expr, lox, env),
        Expr::Unary {
            operator: op,
            expression: expr,
        } => evaluate_unary(expr, op, lox, env),
        Expr::Binary {
            left,
            operator: op,
            right,
        } => evaluate_binary(left, right, op, lox, env),
        Expr::Variable { name } => evaluate_identifier(name, lox, env),
        Expr::Assignment { name, value } => {
            let val = evaluate(value, lox, env)?;
            match env.assign(&name.lexeme, val.clone()) {
                Ok(()) => Some(val),
                Err(str) => {
                    let error = ErrorKind::WithLocation {
                        message: str,
                        line: name.line as u32,
                        col: None,
                    };
                    report(lox, error);
                    None
                }
            }
        }
    }
}

fn evaluate_identifier(identifier: &Token, lox: &mut Lox, env: &Environment) -> Option<Value> {
    match env.lookup(&identifier.lexeme) {
        Some(val) => Some(val.clone()),
        None => {
            report(
                lox,
                ErrorKind::WithLocation {
                    message: format!("Undefined variable '{}'", identifier.lexeme),
                    line: identifier.line as u32,
                    col: None,
                },
            );
            None
        }
    }
}

fn evaluate_unary(
    expression: &Expr,
    operator: &Token,
    lox: &mut Lox,
    env: &mut Environment,
) -> Option<Value> {
    let right = evaluate(expression, lox, env)?;
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

fn evaluate_binary(
    left: &Expr,
    right: &Expr,
    operator: &Token,
    lox: &mut Lox,
    env: &mut Environment,
) -> Option<Value> {
    let left = evaluate(left, lox, env)?;
    let right = evaluate(right, lox, env)?;

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
