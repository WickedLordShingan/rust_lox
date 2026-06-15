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
 - expression only references equality because there is a chain where "element" only references itself and the element just above the precedence level to it
 - why is factor like that ? ? division is left associative meaning a/b/c is a/bc and not ac/b
 - so one of lower precedence on the left na left associativity else right associativity
 - All of this is correct, but the fact that the first symbol in the body of the rule is the same as the head of the rule means this production is left-recursive.
# PRECEDENCE
expression     → ...
equality       → ...
comparison     → ...
term           → ...
factor         → ...
unary          → ...
primary        → ...
`
expression     → equality
equality       → equality ("==" | "!=") comparison | comparison 
comparison     → comparison ( ">" | ">=" | "<" | "<=" ) term | term ;
term           → term ("-" | "+") factor | factor
factor         → factor ("*" | "/") unary | unary
unary          → ("!" | "-") unary | primary
primary        → NUMBER | STRING | "true" | "false" | "nil"
               | "(" expression ")" ;
`
 - instead of the left left-recursive form there is another one that only references the rule just below it.
 - A → A α | β     becomes     A → β α*
 - the above is known as left recursion elimination
`
term → factor (("-" | "+") factor)*
factor → unary (("*" | "/") unary)*
equality → comparison (("==" | "!=") comparison)*
similarly for comparison too
`
 - LL(k), LR(1), LALR—along with more exotic beasts like parser combinators, Earley parsers, the shunting yard algorithm, and packrat parsing.
