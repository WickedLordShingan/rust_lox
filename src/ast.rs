use crate::token::{Literal, Token};

#[derive(Debug)]
pub enum Expr {
    Logical {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Assignment {
        name: Token,
        value: Box<Expr>,
    },
    Variable {
        name: Token,
    },
    Literal {
        value: Option<Literal>,
    },
    Unary {
        operator: Token,
        expression: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
}

#[derive(Debug)]
pub enum Statement {
    ExprStatement(Expr),
    PrintStatement(Expr),
    AssignStatement {
        token: Option<Token>,
        expression: Option<Expr>,
    },
    BlockStatement(Vec<Statement>),
    IfStatement {
        condition: Expr,
        ifblock: Box<Statement>,
        elseblock: Option<Box<Statement>>,
    },
    WhileStatement {
        condition: Expr,
        statement: Box<Statement>,
    },
    // ForStatement {
    //     initializer: Box<Option<Statement>>,
    //     condition: Option<Expr>,
    //     change: Option<Expr>,
    //     statement: Box<Statement>,
    // },
}

pub fn pretty_print(expr: &Expr) -> String {
    match expr {
        Expr::Binary {
            left,
            operator,
            right,
        } => {
            format!(
                "({} {} {})",
                operator.lexeme,
                pretty_print(left),
                pretty_print(right)
            )
        }
        Expr::Grouping { expression } => {
            format!("(group {})", pretty_print(expression))
        }
        Expr::Literal { value } => match value {
            Some(lit) => format!("{:?}", lit),
            None => "nil".to_string(),
        },
        Expr::Unary {
            operator,
            expression,
        } => {
            format!("({} {})", operator.lexeme, pretty_print(expression))
        }
        _ => {
            todo!()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::{Literal, Token, TokenType};

    #[test]
    fn test_pretty_print() {
        let expr = Expr::Binary {
            left: Box::new(Expr::Unary {
                operator: Token::new(TokenType::Minus, "-".to_string(), None, 1),
                expression: Box::new(Expr::Literal {
                    value: Some(Literal::Num(8.0)),
                }),
            }),
            operator: Token::new(TokenType::Star, "*".to_string(), None, 1),
            right: Box::new(Expr::Grouping {
                expression: Box::new(Expr::Binary {
                    left: Box::new(Expr::Literal {
                        value: Some(Literal::Num(3.0)),
                    }),
                    operator: Token::new(TokenType::Plus, "+".to_string(), None, 1),
                    right: Box::new(Expr::Literal {
                        value: Some(Literal::Num(4.0)),
                    }),
                }),
            }),
        };

        let result = pretty_print(&expr);
        println!("{}", result);
        // assert_eq!(result, "(* (- Num(8.0)) (group (+ Num(3.0) Num(4.0))))");
    }
}
