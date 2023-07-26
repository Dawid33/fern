use crate::parser::fern::AstNode;

// This is where we transition from the parser into the ir code
// generation phase. We group all code by function (nested functions
// are outside the cope of this language) and then transform that code
// into static single assignment form.

pub struct Module {
    functions: Vec<Function>,
}

pub struct Function {
    identifiers: Vec<Identifier>,
    stmts: Vec<Statement>,
}

pub struct Identifier {
    name: String,
    // Underscore because type is a keyword in rust
    _type: Type,
}

pub struct Value {
    // There is only one type, the almighty i32 :)
    value: i32,
    _type: Type,
}

enum Type {
    Default,
    I32,
}

pub enum Operation {
    Add,
    Sub,
}

pub enum Expr {
    Binary(Identifier, Operation, Identifier),
    // Constants are represented by an identifier in the symbol table
    Unary(Identifier)
}

pub enum Statement {
    Let(Identifier, Option<Value>),
    Assign(Identifier, Expr),
    Block(Vec<Statement>),
}

impl Module {
    pub fn from(ast: Box<AstNode>) -> Self {
        let functions: Vec<Function> = Vec::new();
        println!("Hello, World");
        return Self {functions};
    }
}
