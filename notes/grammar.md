`
expression     → literal
               | unary
               | binary
               | grouping ;
literal        → NUMBER | STRING | "true" | "false" | "nil" ;
grouping       → "(" expression ")" ;
unary          → ( "-" | "!" ) expression ;
binary         → expression operator expression ;
operator       → "==" | "!=" | "<" | "<=" | ">" | ">="
               | "+"  | "-"  | "*" | "/" ;
`

 - There’s one bit of extra metasyntax here.
 - In addition to quoted strings for terminals that match exact lexemes, we CAPITALIZE terminals that are a single lexeme whose text representation may vary.
 - NUMBER is any number literal, and STRING is any string literal. Later, we’ll do the same for IDENTIFIER.

 - The above grammar is ambiguous

# GEMS
 - each level of precedence gets a rule in the grammar
 - Each rule can only reference itself or rules lower in the table (higher precedence). Never upward.
