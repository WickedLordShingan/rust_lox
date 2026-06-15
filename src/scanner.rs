#![allow(unused)]
use std::collections::HashMap;

use crate::error::{self, Lox, report};
use crate::token::{self, Literal, Token, TokenType};

pub struct Scanner {
    source: String,
    pub tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    keywords: HashMap<String, TokenType>,
}

impl Scanner {
    pub fn init(source: String) -> Scanner {
        let mut keywords = HashMap::new();
        keywords.insert("and".to_string(), TokenType::And);
        keywords.insert("class".to_string(), TokenType::Class);
        keywords.insert("else".to_string(), TokenType::Else);
        keywords.insert("false".to_string(), TokenType::False);
        keywords.insert("for".to_string(), TokenType::For);
        keywords.insert("fun".to_string(), TokenType::Fun);
        keywords.insert("if".to_string(), TokenType::If);
        keywords.insert("nil".to_string(), TokenType::Nil);
        keywords.insert("or".to_string(), TokenType::Or);
        keywords.insert("print".to_string(), TokenType::Print);
        keywords.insert("return".to_string(), TokenType::Return);
        keywords.insert("super".to_string(), TokenType::Super);
        keywords.insert("this".to_string(), TokenType::This);
        keywords.insert("true".to_string(), TokenType::True);
        keywords.insert("var".to_string(), TokenType::Var);
        keywords.insert("while".to_string(), TokenType::While);
        Scanner {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            keywords,
        }
    }

    pub fn scan_tokens(&mut self, lox: &mut Lox) {
        while !(self.is_at_end()) {
            self.start = self.current;
            self.scan_token(lox);
        }

        self.tokens.push(Token {
            token_type: TokenType::Eof,
            lexeme: String::new(),
            literal: Some(Literal::Nil),
            line: self.line,
        });
    }

    fn scan_token(&mut self, lox: &mut Lox) {
        let c = self.advance();
        match c {
            '(' => self.add_token_without_literal(TokenType::LeftParen),
            ')' => self.add_token_without_literal(TokenType::RightParen),
            '{' => self.add_token_without_literal(TokenType::LeftBrace),
            '}' => self.add_token_without_literal(TokenType::RightBrace),
            ',' => self.add_token_without_literal(TokenType::Comma),
            '.' => self.add_token_without_literal(TokenType::Dot),
            '-' => self.add_token_without_literal(TokenType::Minus),
            '+' => self.add_token_without_literal(TokenType::Plus),
            ';' => self.add_token_without_literal(TokenType::Semicolon),
            '*' => self.add_token_without_literal(TokenType::Star),

            '>' => {
                if self.match_next('=') {
                    self.add_token_without_literal(TokenType::GreaterEqual);
                } else {
                    self.add_token_without_literal(TokenType::Greater);
                }
            }
            '<' => {
                if self.match_next('=') {
                    self.add_token_without_literal(TokenType::LessEqual);
                } else {
                    self.add_token_without_literal(TokenType::Less);
                }
            }
            '!' => {
                if self.match_next('=') {
                    self.add_token_without_literal(TokenType::BangEqual);
                } else {
                    self.add_token_without_literal(TokenType::Bang);
                }
            }
            '=' => {
                if self.match_next('=') {
                    self.add_token_without_literal(TokenType::EqualEqual);
                } else {
                    self.add_token_without_literal(TokenType::Equal);
                }
            }

            '/' => {
                if self.peek() == '/' {
                    while self.peek() != '\n' && !(self.is_at_end()) {
                        self.advance();
                    }
                } else {
                    self.add_token_without_literal(TokenType::Slash);
                }
            }

            ' ' => {}
            '\r' => {}
            '\t' => {}
            '\n' => {
                self.line += 1;
            }

            '"' => {
                self.string(lox);
            }

            _ => {
                if is_num(c) {
                    self.number();
                } else if is_alpha(c) {
                    self.identifier(lox);
                } else {
                    lox.had_error = true;
                    report(
                        lox,
                        error::ErrorKind::WithLocation {
                            message: format!("unexpected token {c}"),
                            line: u32::try_from(self.line).unwrap(),
                            col: Some(u32::try_from(self.current).unwrap()),
                        },
                    );
                }
            }
        }
    }

    //HELPERS
    fn number(&mut self) {
        while (is_num(self.peek())) {
            self.advance();
        }
        if (self.peek() == '.' && is_num(self.peek_next())) {
            self.advance();
            while (is_num(self.peek())) {
                self.advance();
            }
        }
        let number = self.source[self.start..self.current]
            .parse::<f64>()
            .unwrap();
        self.add_token_with_literal(TokenType::Number, Literal::Num(number));
    }

    fn identifier(&mut self, lox: &mut Lox) {
        while (is_alpha(self.peek()) || is_num(self.peek())) {
            self.advance();
        }
        let source = &self.source[self.start..self.current];
        if let Some(potential_identifier) = self.keywords.get(source) {
            self.add_token_without_literal(potential_identifier.clone());
        } else {
            self.add_token_without_literal(TokenType::Identifier);
        }
    }

    fn string(&mut self, lox: &mut Lox) {
        while self.peek() != '"' && !(self.is_at_end()) {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        //we hit the end without encountering a second quotes to close the string
        if self.is_at_end() {
            report(
                lox,
                error::ErrorKind::WithLocation {
                    message: String::from("unterminated string literal"),
                    line: u32::try_from(self.line).unwrap(),
                    col: Some(u32::try_from(self.current).unwrap()),
                },
            );
        }

        let string_val = (self.source)[self.start + 1..self.current - 1].to_string();
        self.add_token_with_literal(TokenType::String, Literal::Str(string_val));
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        return self.source.as_bytes()[self.current] as char;
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            return '\0';
        }
        return self.source.as_bytes()[self.current + 1] as char;
    }

    fn match_next(&mut self, expected: char) -> bool {
        if (self.is_at_end()) {
            return false;
        };
        if (self.advance() != expected) {
            return false;
        }
        self.current += 1;
        true
    }

    fn advance(&mut self) -> char {
        let c = self.source.as_bytes()[self.current] as char;
        self.current += 1;
        c
    }

    // ADDING
    fn add_token_without_literal(&mut self, token_type: TokenType) {
        self.add_token_with_literal(token_type, Literal::Nil);
    }

    fn add_token_with_literal(&mut self, token_type: TokenType, literal: Literal) {
        self.tokens.push(Token {
            token_type,
            lexeme: (self.source[self.start..self.current]).to_string(),
            line: self.line,
            literal: Some(literal),
        });
    }
}

fn is_alpha(check: char) -> bool {
    (check >= 'a' && check <= 'z') || (check >= 'A' && check <= 'Z')
}

fn is_num(check: char) -> bool {
    check >= '0' && check <= '9'
}
