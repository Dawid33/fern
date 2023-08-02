use log::info;
use simple_error::SimpleError;

use crate::{fern_ast::TypeExpr, parser::fern_ast::AstNode};
use std::collections::{BTreeMap, VecDeque};

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

pub struct Fn {
    name: String,
    params: Vec<Identifier>,
    body: AstNode,
}

// impl Fn {
//     pub fn from(val: AstNode) -> Result<Self, SimpleError> {
//         Ok()
//     }
// }

#[derive(Eq, PartialOrd, Ord, PartialEq, Hash, Clone)]
pub struct Identifier {
    name: String,
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

pub enum BlockType {
    Module,
    Function,
    If,
    Code,
}

pub struct Block {
    pub block_type: BlockType,
    pub prefix: String,
    pub stable: BTreeMap<Identifier, SymbolData>,
    pub children: Vec<Block>,
}

#[derive(Clone)]
pub enum SymbolType {
    Function,
    Variable,
}

#[derive(Clone)]
pub struct SymbolData {
    symbol_type: SymbolType,
}

impl SymbolData {
    pub fn new(symbol_type: SymbolType) -> Self {
        Self { symbol_type }
    }
}

impl Identifier {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl Block {
    fn new(prefix: String, b_type: BlockType, stable: BTreeMap<Identifier, SymbolData>) -> Self {
        Self {
            block_type: b_type,
            prefix,
            stable,
            children: Vec::new(),
        }
    }

    pub fn from(root: Box<AstNode>) -> Result<Self, SimpleError> {
        let mut root_block = Block::new("root".to_string(), BlockType::Module, BTreeMap::new());
        let stmts = if let AstNode::StatList(list) = *root {
            list
        } else {
            panic!("StatmentList is not root of ast.");
        };
        let mut backlog = VecDeque::new();
        for stmt in stmts {
            match stmt {
                AstNode::Let(_, _, _) => {
                    panic!("Top level statements are not supported.")
                }
                AstNode::Module(_) => panic!("Nested module not supported."),
                AstNode::Function(ref name, _, _) => {
                    if let AstNode::Name(name) = *name.clone() {
                        root_block.stable.insert(Identifier::new(name), SymbolData::new(SymbolType::Function));
                        backlog.push_front(stmt);
                    } else {
                        panic!("Function name must be a valid identifier.");
                    }
                }
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

        for func in backlog {
            root_block.add_func(func);
        }

        return Ok(root_block);
    }

    pub fn add_func(&mut self, val: AstNode) {
        let (name, params, body) = if let AstNode::Function(name, params, body) = val {
            if let AstNode::Name(name) = *name {
                (name, params, body)
            } else {
                panic!("Function name must be a valid identifier.");
            }
        } else {
            panic!("Trying to add function when ast node is not a function.");
        };

        let mut prefix = format!("{}.{}", self.prefix.clone(), name.as_str());
        let mut f = Block::new(prefix, BlockType::Function, self.stable.clone());

        // Add func params to symbol table
        let mut stack = VecDeque::new();
        if let Some(params) = params {
            if let AstNode::ExprList(list) = *params {
                stack.push_front(list);
            }

            while !stack.is_empty() {
                let mut current = stack.pop_front().unwrap();
                let first = current.pop_front().unwrap();
                match first {
                    AstNode::ExprList(list) => {
                        if let AstNode::Name(name) = current.pop_front().unwrap() {
                            f.stable.insert(Identifier::new(name), SymbolData::new(SymbolType::Variable));
                        } else {
                            panic!("Bad ast function params exprlist");
                        }
                        stack.push_front(list);
                    }
                    AstNode::Name(name) => {
                        f.stable.insert(Identifier::new(name), SymbolData::new(SymbolType::Variable));
                        if let AstNode::Name(name) = current.pop_front().unwrap() {
                            f.stable.insert(Identifier::new(name), SymbolData::new(SymbolType::Variable));
                        } else {
                            panic!("Bad ast function params name");
                        }
                    }
                    _ => panic!("Invalid func parameters."),
                }
            }
        }

        if let Some(body) = body {
            if let AstNode::StatList(list) = *body {
                let b = Block::new("".to_string(), BlockType::Code, f.stable.clone());
                f.children.push(b);
            } else {
                panic!("body not statlist");
            }
        }

        self.children.push(f);
    }

    pub fn parse_stmt_list(list: VecDeque<AstNode>) -> VecDeque<Statement> {
        let mut result = VecDeque::new();
        for stmt in list {
            match stmt {
                AstNode::Let(name, type_expr, val) => result.push_front(Block::parse_let(name, type_expr, val)),
                AstNode::Assign(name, val) => (),
                AstNode::Return(val) => (),
                _ => panic!("Invalid statment"),
            }
        }
        return result;
    }

    pub fn parse_let(name: Box<AstNode>, type_expr: Option<TypeExpr>, val: Option<Box<AstNode>>) -> Statement {
        return Statement::Let(Let {});
    }

    pub fn parse_assign(name: Box<AstNode>, val: Option<Box<AstNode>>) {
        info!("parsing assing");
    }

    pub fn add_if(&mut self, val: AstNode) {}
}
