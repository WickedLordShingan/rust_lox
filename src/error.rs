#[derive(Debug)]
pub enum ErrorKind {
    Simple(String),
    WithLocation {
        message: String,
        line: u32,
        col: Option<u32>,
    },
    WithRange {
        message: String,
        start_line: u32,
        end_line: u32,
    },
}

pub struct Lox {
    pub had_error: bool,
}

impl Default for Lox {
    fn default() -> Self {
        Self::new()
    }
}

impl Lox {
    pub fn new() -> Self {
        Self { had_error: false }
    }
}

pub fn report(lox: &mut Lox, error: ErrorKind) {
    match &error {
        ErrorKind::Simple(message) => {
            eprintln!("{message}");
        }
        ErrorKind::WithLocation { message, line, col } => match col {
            Some(col) => eprintln!("[line {line}, col {col}] Error: {message}"),
            None => eprintln!("[line {line}] Error: {message}"),
        },
        ErrorKind::WithRange {
            message,
            start_line,
            end_line,
        } => {
            eprintln!("[lines {start_line}–{end_line}] Error: {message}");
        }
    }
    lox.had_error = true;
}
