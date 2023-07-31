use crate::parser::fern::AstNode;
use std::collections::HashMap;

// This is where we transition from the parser into the ir code
// generation phase. We group all code by function (nested functions
// are outside the cope of this language) and then transform that code
// into static single assignment form.

pub struct Module {
    top_level_stmts: Vec<Statement>,
}

pub enum Statement {
    Fn(Fn),
    If(If),
    Let(Let),
    Assign(Assign),
}

pub struct Assign {}

pub struct Let {}

pub struct If {}

pub struct Fn {}

#[derive(Eq, PartialEq, Hash)]
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

#[derive(Eq, PartialEq, Hash)]
enum Type {
    Default,
    I32,
}

pub enum Operation {
    Add,
    Sub,
}

impl Module {
    pub fn new() -> Self {
        return Self {
            top_level_stmts: Vec::new(),
        };
    }

    pub fn from(root: Box<AstNode>) -> Self {
        let mut module = Module::new();
        let mut symbol_table: HashMap<Identifier, ()> = HashMap::new();
        let mut backlog: Vec<AstNode> = Vec::new();
        let stmts = if let AstNode::StatList(list) = *root {
            list
        } else {
            panic!("StatmentList is not root of ast.");
        };
        for stmt in stmts {
            match stmt {
                AstNode::Let(identifier, type_expr, value) => {}
                AstNode::Module(_) => panic!("Nested statements not supported."),
                AstNode::Function(_, _, _) => backlog.push(stmt),
                AstNode::Binary(_, _, _)
                | AstNode::Unary(_, _)
                | AstNode::Number(_)
                | AstNode::String(_)
                | AstNode::Name(_)
                | AstNode::ExprList(_)
                | AstNode::Assign(_, _)
                | AstNode::Return(_)
                | AstNode::StatList(_)
                | AstNode::If(_, _, _)
                | AstNode::ExprThen(_, _)
                | AstNode::ElseIf(_, _, _)
                | AstNode::Else(_)
                | AstNode::For(_, _, _)
                | AstNode::While(_, _) => panic!("Bad top level stmt. Should be let, function or module."),
            }
        }
        return module;
    }
}
