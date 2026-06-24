#![allow(unused)]

use std::fmt::Arguments;

use crate::ast::{Expr, Statement};
use crate::environment::{self, Environment};
use crate::error::{ErrorKind, Lox, report};
use crate::token::{self, Token, TokenType};
use crate::value::{self, *};

pub fn interpret(statements: &Vec<Statement>, lox: &mut Lox, environment: &mut Environment) {
    for stmt in statements {
        execute(stmt, lox, environment);
        //need to stop if error
    }
}

pub fn execute(stmt: &Statement, lox: &mut Lox, env: &mut Environment) -> Option<Value> {
    match stmt {
        Statement::PrintStatement(expr) => {
            // println!("{:?}", expr);
            if let Some(value) = evaluate(expr, lox, env) {
                println!("{value}");
            }
            None
        }

        Statement::ExprStatement(expr) => {
            evaluate(expr, lox, env);
            None
        }

        Statement::AssignStatement { token, expression } => {
            var_declaration(token, expression, lox, env);
            None
        }

        Statement::BlockStatement(statements) => {
            env.start_scope();
            for statement in statements {
                if let Some(return_val) = execute(statement, lox, env) {
                    return Some(return_val);
                }
                if lox.had_error {
                    break;
                }
            }
            env.end_scope();
            None
        }

        Statement::IfStatement {
            condition,
            ifblock,
            elseblock,
        } => {
            if let Some(condition_result) = evaluate(condition, lox, env) {
                if is_truthy(&condition_result) {
                    return execute(ifblock, lox, env);
                } else if let Some(else_block) = elseblock {
                    return execute(else_block, lox, env);
                }
            }
            None
        }

        Statement::WhileStatement {
            condition,
            statement,
        } => {
            if let Some(mut evaluated_condition) = evaluate(condition, lox, env) {
                while is_truthy(&evaluated_condition) {
                    if let Some(return_val) = execute(statement, lox, env) {
                        return Some(return_val);
                    }
                    match evaluate(condition, lox, env) {
                        Some(val) => evaluated_condition = val, // update it
                        None => break,
                    }
                }
            }
            None
            // while evaluate(condition, lox, env)
            //     .map(|v| is_truthy(&v))
            //     .unwrap_or(false)
            // {
            //     execute(statement, lox, env);
            // }
        }

        Statement::FunctionDeclaration {
            name,
            parameters,
            body,
        } => {
            env.define(
                &name.lexeme,
                Value::Function {
                    name: name.lexeme.clone(),
                    params: parameters.clone(),
                    body: body.clone(),
                },
            );
            None
        }

        Statement::ReturnStatement { token, expr } => {
            let val = match expr {
                Some(expression) => evaluate(expression, lox, env).unwrap_or(Value::Nil),
                None => Value::Nil,
            };
            Some(val)
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
        Expr::Logical {
            left,
            operator,
            right,
        } => evaluate_logical(left, right, operator, lox, env),
        Expr::Call {
            callee,
            arguments,
            paren,
        } => evaluate_call(callee, arguments, paren, lox, env),
    }
}

fn evaluate_call(
    callee: &Expr,
    arguments: &[Expr],
    paren: &Token,
    lox: &mut Lox,
    env: &mut Environment,
) -> Option<Value> {
    let callee = evaluate(callee, lox, env)?;
    if let Value::Function { name, params, body } = callee {
        return call_function(paren, &name, &params, &body, arguments, lox, env);
    }
    let error = ErrorKind::WithLocation {
        message: String::from("Cant call random shit bruh need a fucking function"),
        line: paren.line as u32,
        col: None,
    };
    report(lox, error);
    None
}

fn call_function(
    paren: &Token,
    name: &str,
    parameters: &[Token],
    body: &[Statement],
    arguments: &[Expr],
    lox: &mut Lox,
    env: &mut Environment,
) -> Option<Value> {
    if parameters.len() != arguments.len() {
        let error = ErrorKind::WithLocation {
            message: format!(
                "Function {} expected {} arguments but got {} arguments",
                name,
                parameters.len(),
                arguments.len(),
            ),
            line: paren.line as u32,
            col: None,
        };
        report(lox, error);
        return None;
    }

    let evaluated: Vec<Value> = arguments
        .iter()
        .map(|arg| evaluate(arg, lox, env))
        .collect::<Option<Vec<_>>>()?; // any None short-circuits the whole thing

    env.start_scope();

    for (param, value) in parameters.iter().zip(evaluated) {
        env.define(&param.lexeme, value);
    }

    let return_val = {
        let mut result = None;
        for stmt in body {
            if let Some(val) = execute(stmt, lox, env) {
                result = Some(val);
                break;
            }
            if lox.had_error {
                break;
            }
        }
        result
    };

    env.end_scope();
    return_val.or(Some(Value::Nil))
}

fn evaluate_logical(
    left: &Expr,
    right: &Expr,
    operator: &Token,
    lox: &mut Lox,
    env: &mut Environment,
) -> Option<Value> {
    match operator.token_type {
        TokenType::Or => {
            let left = evaluate(left, lox, env)?;
            if (is_truthy(&left)) {
                return Some(left);
            }
            let right = evaluate(right, lox, env)?;
            Some(right)
        }
        TokenType::And => {
            let left = evaluate(left, lox, env)?;
            if (!is_truthy(&left)) {
                return Some(left);
            }
            let right = evaluate(right, lox, env)?;
            Some(right)
        }
        _ => None,
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
        Value::Function { name, params, body } => todo!(),
    }
}
