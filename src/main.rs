mod ast;
mod error;
mod interpreter;
mod parser;
mod running;
mod scanner;
mod token;
mod value;

use error::Lox;
use running::{runfile, runprompt};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut lox = Lox::new();
    if args.len() > 1 {
        runfile(&mut lox, &args[1]);
        if lox.had_error {
            std::process::exit(65);
        }
    } else {
        runprompt(&mut lox);
    }
}
