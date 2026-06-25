#![allow(unused)]

use std::fmt::Arguments;

use crate::ast::{Expr, Statement};
use crate::environment::{self, Environment};
use crate::error::{ErrorKind, Lox, report};
use crate::token::{self, Token, TokenType};
use crate::value::{self, *};

pub struct Interpreter {
    globals: Environment,
    current_env: Environment,
}

impl Interpreter {
    pub fn init() -> Self {
        Self {
            globals: Environment::init(),
            current_env: Environment::init(),
        }
    }

    pub fn interpret(&mut self, statements: &Vec<Statement>, lox: &mut Lox) {
        for stmt in statements {
            self.execute(stmt, lox);
            //need to stop if error
            if (lox.had_error) {
                break;
            }
        }
    }

    pub fn execute(&mut self, stmt: &Statement, lox: &mut Lox) -> Option<Value> {
        match stmt {
            Statement::PrintStatement(expr) => {
                // println!("{:?}", expr);
                if let Some(value) = self.evaluate(expr, lox) {
                    println!("{value}");
                }
                None
            }

            Statement::ExprStatement(expr) => {
                self.evaluate(expr, lox);
                None
            }

            Statement::AssignStatement { token, expression } => {
                self.var_declaration(token, expression, lox);
                None
            }

            Statement::BlockStatement(statements) => {
                // let mut returny = None;
                self.current_env.start_scope();
                for statement in statements {
                    if let Some(return_val) = self.execute(statement, lox) {
                        return Some(return_val);
                    }
                    if lox.had_error {
                        break;
                    }
                }
                self.current_env.end_scope();
                None
            }

            Statement::IfStatement {
                condition,
                ifblock,
                elseblock,
            } => {
                if let Some(condition_result) = self.evaluate(condition, lox) {
                    if is_truthy(&condition_result) {
                        return self.execute(ifblock, lox);
                    } else if let Some(else_block) = elseblock {
                        return self.execute(else_block, lox);
                    }
                }
                None
            }

            Statement::WhileStatement {
                condition,
                statement,
            } => {
                if let Some(mut evaluated_condition) = self.evaluate(condition, lox) {
                    while is_truthy(&evaluated_condition) {
                        if let Some(return_val) = self.execute(statement, lox) {
                            return Some(return_val);
                        }
                        match self.evaluate(condition, lox) {
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
                self.current_env.define(
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
                    Some(expression) => self.evaluate(expression, lox).unwrap_or(Value::Nil),
                    None => Value::Nil,
                };
                Some(val)
            }
        }
    }

    fn var_declaration(&mut self, token: &Option<Token>, expression: &Option<Expr>, lox: &mut Lox) {
        let value = match expression {
            Some(expr) => self.evaluate(expr, lox).unwrap_or(Value::Nil),
            None => Value::Nil,
        };
        if let Some(tok) = token {
            self.current_env.define(&tok.lexeme, value);
        }
    }

    fn evaluate(&mut self, expression: &Expr, lox: &mut Lox) -> Option<Value> {
        match expression {
            Expr::Literal { value: literal } => Some(Value::from(literal.clone()?)),
            Expr::Grouping { expression: expr } => self.evaluate(expr, lox),
            Expr::Unary {
                operator: op,
                expression: expr,
            } => self.evaluate_unary(expr, op, lox),
            Expr::Binary {
                left,
                operator: op,
                right,
            } => self.evaluate_binary(left, right, op, lox),
            Expr::Variable { name } => self.evaluate_identifier(name, lox),
            Expr::Assignment { name, value } => {
                let val = self.evaluate(value, lox)?;
                match self.current_env.assign(&name.lexeme, val.clone()) {
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
            } => self.evaluate_logical(left, right, operator, lox),
            Expr::Call {
                callee,
                arguments,
                paren,
            } => self.evaluate_call(callee, arguments, paren, lox),
        }
    }

    fn evaluate_call(
        &mut self,
        callee: &Expr,
        arguments: &[Expr],
        paren: &Token,
        lox: &mut Lox,
    ) -> Option<Value> {
        let callee = self.evaluate(callee, lox)?;
        if let Value::Function { name, params, body } = callee {
            return self.call_function(paren, &name, &params, &body, arguments, lox);
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
        &mut self,
        paren: &Token,
        name: &str,
        parameters: &[Token],
        body: &[Statement],
        arguments: &[Expr],
        lox: &mut Lox,
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
            .map(|arg| self.evaluate(arg, lox))
            .collect::<Option<Vec<_>>>()?; // any None short-circuits the whole thing

        // let global_clone = self.globals.clone();
        // let old_env = self.current_env.replace_with(global_clone);
        self.current_env.start_scope();

        for (param, value) in parameters.iter().zip(evaluated) {
            self.current_env.define(&param.lexeme, value);
        }

        let return_val = {
            let mut result = None;
            for stmt in body {
                if let Some(val) = self.execute(stmt, lox) {
                    result = Some(val);
                    break;
                }
                if lox.had_error {
                    break;
                }
            }
            result
        };

        self.current_env.end_scope();
        // let _ = self.current_env.replace_with(old_env);
        return_val.or(Some(Value::Nil))
    }

    fn evaluate_logical(
        &mut self,
        left: &Expr,
        right: &Expr,
        operator: &Token,
        lox: &mut Lox,
    ) -> Option<Value> {
        match operator.token_type {
            TokenType::Or => {
                let left = self.evaluate(left, lox)?;
                if (is_truthy(&left)) {
                    return Some(left);
                }
                let right = self.evaluate(right, lox)?;
                Some(right)
            }
            TokenType::And => {
                let left = self.evaluate(left, lox)?;
                if (!is_truthy(&left)) {
                    return Some(left);
                }
                let right = self.evaluate(right, lox)?;
                Some(right)
            }
            _ => None,
        }
    }

    fn evaluate_identifier(&mut self, identifier: &Token, lox: &mut Lox) -> Option<Value> {
        // println!("looking up: {}", identifier.lexeme);
        match self.current_env.lookup(&identifier.lexeme) {
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
        &mut self,
        expression: &Expr,
        operator: &Token,
        lox: &mut Lox,
    ) -> Option<Value> {
        let right = self.evaluate(expression, lox)?;
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
        &mut self,
        left: &Expr,
        right: &Expr,
        operator: &Token,
        lox: &mut Lox,
    ) -> Option<Value> {
        let left = self.evaluate(left, lox)?;
        let right = self.evaluate(right, lox)?;

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
