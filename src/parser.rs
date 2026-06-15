#![allow(unused)]

use crate::ast::Expr;
use crate::error::{ErrorKind, Lox, report};
use crate::token::{self, Literal, Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn init(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn expression(&mut self, lox: &mut Lox) -> Expr {
        self.equality(lox)
    }

    fn equality(&mut self, lox: &mut Lox) -> Expr {
        let mut expr = self.comparison(lox);
        while (self.match_types(vec![TokenType::BangEqual, TokenType::EqualEqual])) {
            let operation = self.previous_token().unwrap().clone();
            let inner = self.comparison(lox);
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operation,
                right: Box::new(inner),
            };
        }
        expr
    }

    fn comparison(&mut self, lox: &mut Lox) -> Expr {
        let mut expr = self.term(lox);
        while (self.match_types(vec![
            TokenType::Less,
            TokenType::LessEqual,
            TokenType::Greater,
            TokenType::GreaterEqual,
        ])) {
            let operation = self.previous_token().unwrap().clone();
            let inner = self.term(lox);
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operation,
                right: Box::new(inner),
            };
        }
        expr
    }

    fn term(&mut self, lox: &mut Lox) -> Expr {
        let mut expr = self.factor(lox);
        while (self.match_types(vec![TokenType::Plus, TokenType::Minus])) {
            let operation = self.previous_token().unwrap().clone();
            let inner = self.factor(lox);
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operation,
                right: Box::new(inner),
            };
        }
        expr
    }

    fn factor(&mut self, lox: &mut Lox) -> Expr {
        let mut expr = self.unary(lox);
        while (self.match_types(vec![TokenType::Star, TokenType::Slash])) {
            let operation = self.previous_token().unwrap().clone();
            let inner = self.unary(lox);
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: operation,
                right: Box::new(inner),
            };
        }
        expr
    }

    fn unary(&mut self, lox: &mut Lox) -> Expr {
        if self.match_types(vec![TokenType::Bang, TokenType::Minus]) {
            let operation = self.previous_token().unwrap().clone();
            let mut inner = self.unary(lox);
            inner = Expr::Unary {
                operator: operation,
                expression: Box::new(inner),
            };
            return inner;
        } else {
            return self.primary(lox);
        }
    }

    fn primary(&mut self, lox: &mut Lox) -> Expr {
        if self.match_types(vec![
            TokenType::Number,
            TokenType::String,
            TokenType::True,
            TokenType::False,
            TokenType::Nil,
        ]) {
            return Expr::Literal {
                value: self.previous_token().unwrap().literal.clone(),
            };
        }

        if self.match_types(vec![TokenType::LeftParen]) {
            let expr = self.equality(lox);
            self.consume(lox, TokenType::RightParen, "Expect ')' after expression.");
            return Expr::Grouping {
                expression: Box::new(expr),
            };
        }

        let token = self.peek().unwrap();
        report(
            lox,
            ErrorKind::WithLocation {
                message: "Expect expression.".to_string(),
                line: token.line as u32,
                col: None,
            },
        );

        Expr::Literal { value: None }
    }

    //helpers
    fn consume(&mut self, lox: &mut Lox, token_type: TokenType, message: &str) -> Option<&Token> {
        if self.check(&token_type) {
            return self.advance();
        }
        let token = self.peek().unwrap();
        report(
            lox,
            ErrorKind::WithLocation {
                message: message.to_string(),
                line: token.line as u32,
                col: None,
            },
        );
        None
    }

    fn match_types(&mut self, types: Vec<TokenType>) -> bool {
        for check_type in types.iter() {
            if (self.check(check_type)) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, check_type: &TokenType) -> bool {
        if (self.is_at_end()) {
            return false;
        }
        self.peek().unwrap().token_type == *check_type
    }

    fn is_at_end(&self) -> bool {
        if self.current >= self.tokens.len() {
            return true;
        }
        false
    }

    fn advance(&mut self) -> Option<&Token> {
        if !(self.is_at_end()) {
            self.current += 1;
            return self.previous_token();
        }
        None
    }

    fn peek(&self) -> Option<&Token> {
        if self.is_at_end() {
            return None;
        }
        Some(&self.tokens[self.current])
    }

    fn previous_token(&self) -> Option<&Token> {
        if self.current == 0 || self.current > self.tokens.len() {
            return None;
        }
        Some(&self.tokens[self.current - 1])
    }
}
