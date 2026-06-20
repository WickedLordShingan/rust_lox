#![allow(unused)]

use crate::ast::{Statement, pretty_print};
use crate::error::{ErrorKind, Lox, report};
use crate::interpreter::interpret;
use crate::parser::Parser;
use crate::scanner::Scanner;
use std::fs;
use std::io::{self, BufRead, Write};

pub fn runfile(lox: &mut Lox, filename: &str) {
    match fs::read_to_string(filename) {
        Ok(source) => run(lox, &source),
        Err(e) => report(lox, ErrorKind::Simple(e.to_string())),
    }
}

pub fn runprompt(lox: &mut Lox) {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
                run(lox, line.trim());
                lox.had_error = false;
            }
            Err(e) => {
                report(lox, ErrorKind::Simple(e.to_string()));
                break;
            }
        }
    }
}

fn run(lox: &mut Lox, source: &str) {
    let mut scanner = Scanner::init(source.to_string());
    scanner.scan_tokens(lox);

    if lox.had_error {
        return;
    }

    // println!("{:?}", scanner.tokens);
    let mut parser = Parser::init(scanner.tokens);

    let stmts = parser.parse(lox);
    // println!("{:?}", stmts);

    if lox.had_error {
        return;
    }

    interpret(&stmts, lox);
}
