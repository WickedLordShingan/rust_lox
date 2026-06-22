#![allow(unused)]

use std::vec;

use crate::ast::{Expr, Statement};
use crate::error::{self, ErrorKind, Lox, report};
use crate::token::{self, Literal, Token, TokenType};
use crate::value::{self, Value};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn init(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self, lox: &mut Lox) -> Vec<Statement> {
        let mut statements = Vec::new();
        while !(self.is_at_end()) {
            if let Some(statement) = self.declaration(lox) {
                statements.push(statement);
            }
        }
        statements
    }

    fn declaration(&mut self, lox: &mut Lox) -> Option<Statement> {
        if self.match_types(vec![TokenType::Var]) {
            return Some(self.var_declaration(lox));
        }
        let temp = self.statement(lox);
        if lox.had_error {
            self.synchronize();
            lox.had_error = false;
            return None;
        }
        Some(temp)
    }

    fn var_declaration(&mut self, lox: &mut Lox) -> Statement {
        let name = self
            .consume(lox, TokenType::Identifier, "Expected a variable name")
            .cloned();
        let mut initializer = None;
        if (self.match_types(vec![TokenType::Equal])) {
            initializer = Some(self.expression(lox));
        }
        self.consume(
            lox,
            TokenType::Semicolon,
            "Expect ';' after variable declaration.",
        );
        Statement::AssignStatement {
            token: name,
            expression: initializer,
        }
    }

    fn statement(&mut self, lox: &mut Lox) -> Statement {
        if self.match_types(vec![TokenType::Print]) {
            return self.print_statement(lox);
        }
        if self.match_types(vec![TokenType::LeftBrace]) {
            return Statement::BlockStatement(self.block_statement(lox));
        }
        if self.match_types(vec![TokenType::If]) {
            return self.if_statement(lox);
        }
        if self.match_types(vec![TokenType::While]) {
            return self.while_statement(lox);
        }
        if self.match_types(vec![TokenType::For]) {
            return self.for_statement(lox);
        }
        self.expression_statement(lox)
    }

    fn for_statement(&mut self, lox: &mut Lox) -> Statement {
        self.consume(
            lox,
            TokenType::LeftParen,
            "Open paren was not found after the for keyword",
        );

        let mut initializer: Option<Statement>;
        if self.match_types(vec![TokenType::Semicolon]) {
            initializer = None;
        } else if self.match_types(vec![TokenType::Var]) {
            initializer = Some(self.var_declaration(lox));
        } else {
            initializer = Some(self.expression_statement(lox));
        }

        let mut condition: Option<Expr> = None;
        if (!self.check(&TokenType::Semicolon)) {
            condition = Some(self.expression(lox));
        }
        self.consume(
            lox,
            TokenType::Semicolon,
            "Expected semicolon after initializer in for loop",
        );

        let mut change: Option<Expr> = None;
        if (!self.check(&TokenType::RightParen)) {
            change = Some(self.expression(lox));
        }
        self.consume(
            lox,
            TokenType::RightParen,
            "Expected closing paren after condition in for loop",
        );

        let body = self.statement(lox);

        let mut statements = Vec::new();
        if let Some(stmt) = initializer {
            statements.push(stmt);
        }

        let mut body_and_change = vec![body];
        if let Some(change) = change {
            body_and_change.push(Statement::ExprStatement(change));
        }

        statements.push(Statement::WhileStatement {
            condition: condition.unwrap_or(Expr::Literal {
                value: Some(Literal::Bool(true)),
            }),
            statement: Box::new(Statement::BlockStatement(body_and_change)),
        });

        Statement::BlockStatement(statements)
    }

    fn while_statement(&mut self, lox: &mut Lox) -> Statement {
        self.consume(
            lox,
            TokenType::LeftParen,
            "Open paren was not found after the for keyword",
        );
        let condition = self.expression(lox);
        self.consume(lox, TokenType::RightParen, "Closing paren was not found");
        let statement = Box::new(self.statement(lox));
        Statement::WhileStatement {
            condition,
            statement,
        }
    }

    fn if_statement(&mut self, lox: &mut Lox) -> Statement {
        self.consume(
            lox,
            TokenType::LeftParen,
            "Open paren was not found after the if keyword",
        );
        let condition = self.expression(lox);
        self.consume(lox, TokenType::RightParen, "Closing paren was not found");
        let ifblock = Box::new(self.statement(lox));
        let mut elseblock: Option<Box<Statement>> = None;
        if (self.match_types(vec![TokenType::Else])) {
            elseblock = Some(Box::new(self.statement(lox)));
        }
        Statement::IfStatement {
            condition,
            ifblock,
            elseblock,
        }
    }

    fn block_statement(&mut self, lox: &mut Lox) -> Vec<Statement> {
        let mut statements: Vec<Statement> = Vec::new();
        while !self.check(&TokenType::RightBrace) {
            if let Some(statement) = self.declaration(lox) {
                statements.push(statement);
            }
        }
        self.consume(lox, TokenType::RightBrace, "Expected '}' after a block");
        statements
    }

    fn print_statement(&mut self, lox: &mut Lox) -> Statement {
        let expr = self.expression(lox);
        self.consume(lox, TokenType::Semicolon, "Expect ';' after value.");
        Statement::PrintStatement(expr)
    }

    fn expression_statement(&mut self, lox: &mut Lox) -> Statement {
        let expr = self.expression(lox);
        self.consume(lox, TokenType::Semicolon, "Expect ';' after expression.");
        Statement::ExprStatement(expr)
    }

    //expression
    pub fn expression(&mut self, lox: &mut Lox) -> Expr {
        self.assignment(lox)
    }

    fn assignment(&mut self, lox: &mut Lox) -> Expr {
        let variable = self.logical_or(lox);
        if (self.match_types(vec![TokenType::Equal])) {
            let line_no = self.previous_token().unwrap().line;
            let remaining = self.assignment(lox);
            if let Expr::Variable { name } = variable {
                return Expr::Assignment {
                    name,
                    value: Box::new(remaining),
                };
            }
            let error = ErrorKind::WithLocation {
                message: String::from("Cant assign to something that is not a variable"),
                line: line_no as u32,
                col: None,
            };
            report(lox, error);
        }
        variable
    }

    fn logical_or(&mut self, lox: &mut Lox) -> Expr {
        let mut expr = self.logical_and(lox);
        while (self.match_types(vec![TokenType::Or])) {
            let operation = self.previous_token().unwrap().clone();
            let inner = self.logical_and(lox);
            expr = Expr::Logical {
                left: Box::new(expr),
                operator: operation,
                right: Box::new(inner),
            };
        }
        expr
    }

    fn logical_and(&mut self, lox: &mut Lox) -> Expr {
        let mut expr = self.equality(lox);
        while (self.match_types(vec![TokenType::And])) {
            let operation = self.previous_token().unwrap().clone();
            let inner = self.equality(lox);
            expr = Expr::Logical {
                left: Box::new(expr),
                operator: operation,
                right: Box::new(inner),
            };
        }
        expr
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
            inner
        } else {
            self.primary(lox)
        }
    }

    fn primary(&mut self, lox: &mut Lox) -> Expr {
        if self.match_types(vec![TokenType::Number, TokenType::String, TokenType::Nil]) {
            return Expr::Literal {
                value: self.previous_token().unwrap().literal.clone(),
            };
        }

        if self.match_types(vec![TokenType::False]) {
            return Expr::Literal {
                value: Some(Literal::Bool(false)),
            };
        }

        if self.match_types(vec![TokenType::True]) {
            return Expr::Literal {
                value: Some(Literal::Bool(true)),
            };
        }

        if self.match_types(vec![TokenType::Identifier]) {
            return Expr::Variable {
                name: self.previous_token().cloned().unwrap(),
            };
        }

        if self.match_types(vec![TokenType::LeftParen]) {
            let expr = self.expression(lox);
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

    fn synchronize(&mut self) {
        while !(self.is_at_end()) {
            if (self
                .previous_token()
                .is_some_and(|t| t.token_type == TokenType::Semicolon))
            {
                return;
            }
            match self.peek().map(|t| &t.token_type) {
                Some(TokenType::Class)
                | Some(TokenType::Fun)
                | Some(TokenType::Var)
                | Some(TokenType::For)
                | Some(TokenType::If)
                | Some(TokenType::While)
                | Some(TokenType::Print)
                | Some(TokenType::Return) => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn consume(&mut self, lox: &mut Lox, token_type: TokenType, message: &str) -> Option<&Token> {
        if self.check(&token_type) {
            return self.advance();
        }
        let line = self
            .peek()
            .map_or(self.tokens.last().unwrap().line, |t| t.line);

        report(
            lox,
            ErrorKind::WithLocation {
                message: message.to_string(),
                line: line as u32,
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
        if self.current >= self.tokens.len()
            || self.tokens[self.current].token_type == TokenType::Eof
        {
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
        if self.current == 0 || (self.current > self.tokens.len()) {
            return None;
        }
        Some(&self.tokens[self.current - 1])
    }
}
