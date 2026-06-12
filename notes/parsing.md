A recursive descent parser is a literal translation of the grammar’s rules straight into imperative code. Each rule becomes a function. The body of the rule translates to code roughly like:

Grammar notation         Code representation

Terminal                 Code to match and consume a token
Nonterminal              Call to that rule’s function
|                        if or switch statement
* or +                   while or for loop
?                        if statement
