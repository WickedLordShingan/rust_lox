A recursive descent parser is a literal translation of the grammar’s rules straight into imperative code. Each rule becomes a function. The body of the rule translates to code roughly like:

Grammar notation         Code representation

Terminal                 Code to match and consume a token
Nonterminal              Call to that rule’s function
|                        if or switch statement
* or +                   while or for loop
?                        if statement

- even though our grammar changed the variants of expression stay the same (binary, unary, literal, grouping)
 - we need more helpers just like the lexer (like peek and advance)
 - and the functions here consume tokens and return an expression

# ERROR HANDLING
 - be fast
 - dont quit after the first error
 - Minimize cascaded errors. errors caused by just one error. For example
`
  x = 0
  x.as_string()
  x.len()
  x.iter()
`
 - In Lox, values are created by literals, computed by expressions, and stored in variables.
