**Checkpoint 1: Print and Expression Statements**

The grammar changed from "parse one expression" to "parse a list of statements." 

*What you added:*
- `Statement` enum with `PrintStatement(Expr)` and `ExprStatement(Expr)`
- `parse()` — loops calling `declaration()` until end
- `statement()` — checks for `print` keyword, otherwise expression statement
- `print_statement()` — parses expression, expects `;`, returns `Statement::PrintStatement`
- `expression_statement()` — parses expression, expects `;`, returns `Statement::ExprStatement`

*Interpreter:*
- `execute()` — matches on `Statement`, for `PrintStatement` evaluates and prints, for `ExprStatement` just evaluates and throws away result
- `interpret()` — loops calling `execute` on each statement

---

**Checkpoint 2: Variable Declaration**

Added the ability to create new variables with `var`.

*What you added:*
- `Statement::VarDeclaration { token: Option<Token>, expression: Option<Expr> }`
- `Environment` struct with `HashMap<String, Value>`, `define`, `lookup`, `assign`
- `declaration()` — sits above `statement()` in the call chain, checks for `var` keyword
- `var_declaration()` — consumes identifier, optionally consumes `=` and expression, expects `;`

*Interpreter:*
- `execute` gets `&mut Environment` threaded through
- `VarDeclaration` arm — evaluates initializer if present (else `Value::Nil`), calls `env.define(&token.lexeme, value)`

---

**Checkpoint 3: Variable Access**

Added the ability to *read* a variable's value inside an expression.

*What you added:*
- `Expr::Variable { name: Token }` in ast.rs
- `primary()` — new arm that matches `TokenType::Identifier` and returns `Expr::Variable { name: token }`

*Interpreter:*
- New arm in `evaluate` for `Expr::Variable`:
```rust
Expr::Variable { name } => {
    match env.lookup(&name.lexeme) {
        Some(val) => Some(val.clone()),
        None => {
            report(lox, ErrorKind::WithLocation {
                message: format!("Undefined variable '{}'", name.lexeme),
                line: name.line as u32,
                col: None,
            });
            None
        }
    }
}
```

---

**Checkpoint 4: Assignment**

Added the ability to *reassign* an existing variable.

*What you added:*
- `Expr::Assignment { name: Token, value: Box<Expr> }` in ast.rs
- `assignment()` in parser — sits between `expression()` and `equality()` in the chain:
  - parses left side as `equality()`
  - if `=` follows, recursively parses right side as `assignment()` (right-associative)
  - checks if left side was `Expr::Variable` — if yes, returns `Expr::Assignment`, if no, reports error
- `expression()` now calls `assignment()` instead of `equality()`
- `env.assign()` — like `define` but errors if variable doesn't exist yet

*Interpreter:*
- New arm in `evaluate` for `Expr::Assignment`:
```rust
Expr::Assignment { name, value } => {
    let val = evaluate(value, lox, env)?;
    match env.assign(&name.lexeme, val.clone()) {
        Ok(()) => Some(val),  // return value so a = b = 3 propagates
        Err(msg) => {
            report(lox, ErrorKind::WithLocation {
                message: msg,
                line: name.line as u32,
                col: None,
            });
            None
        }
    }
}
```

