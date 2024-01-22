## After-break clarity
Duplicate variable identifier checking

## Lexer / Parser
Make lexer lex ==, <= and >=
Add type expressions to parser 
Made AstNode correctly parse expression with brackets.
Write while and for loop parsing in AstNode

## Ir / Codegen
Destructure expressions into static single assignment.
Assign labels and link function calls, variables.
Register allocation
Final codegen

### Nice to have's
Write only global symbol table for constants

## Stretch goals
Basic type system
VCS utilizing ast of language to resolve merges
VCS based on patch theory ala pijul

Testing
- Using selectors (ala CSS selectors) as assertions in tests.
- Binary level branch test coverage, metrics and location.
