use log::info;
use simple_error::SimpleError;

use crate::{
    fern_ast::{Operator, TypeExpr},
    parser::fern_ast::AstNode,
};
use std::{
    collections::{BTreeMap, VecDeque},
    fmt::Display,
};

// This is where we transition from the parser into the ir code
// generation phase. We group all code by function (nested functions
// are outside the cope of this language) and then transform that code
// into static single assignment form.

pub struct Module {
    top_level_stmts: Vec<Statement>,
}

#[derive(Debug)]
pub enum Statement {
    Let(Let),
    Goto(Identifier),
    Return(Option<Value>),
}

#[derive(Debug)]
pub struct Assign {}

#[derive(Debug)]
pub enum Expr {
    Binary(Value, Operator, Value),
    Unary(Operator, Value),
    Single(Value),
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Binary(left, op, right) => write!(f, "{} {:?} {}", left, op, right),
            Expr::Unary(op, right) => write!(f, "{:?} {}", op, right),
            Expr::Single(x) => write!(f, "{}", x),
        }
    }
}

#[derive(Debug)]
pub enum Value {
    Identifier(String),
    Number(i64),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Identifier(x) => write!(f, "{}", x),
            Value::Number(x) => write!(f, "{}", x),
        }
    }
}

#[derive(Debug)]
pub struct Let {
    pub ident: Identifier,
    pub val: Option<Expr>,
}

impl Let {
    pub fn new(ident: Identifier, val: Option<Expr>) -> Self {
        Self { ident, val }
    }
}

#[derive(Debug)]
pub struct If {}

#[derive(Debug)]
pub struct Fn {
    name: String,
    params: Vec<Identifier>,
    body: AstNode,
}

#[derive(Eq, PartialOrd, Ord, PartialEq, Hash, Clone, Debug)]
pub struct Identifier {
    pub name: String,
}

#[derive(Eq, PartialEq, Hash, Debug)]
enum Type {
    Default,
    I32,
}

pub enum BlockType {
    Module,
    Function,
    If(Value),
    ElseIf,
    Else,
    Code(VecDeque<Statement>),
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
    Constant,
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
        let stmts = if let AstNode::Module(statlist) = *root {
            if let AstNode::StatList(list) = *statlist {
                list
            } else {
                panic!("Malformed module");
            }
        } else {
            panic!("Module is not root of ast.");
        };

        let mut backlog = VecDeque::new();
        for stmt in stmts {
            match stmt {
                AstNode::Let(_, _, _) => {
                    panic!("Top level statements are not supported.")
                }
                AstNode::Module(_) => panic!("Nested module not supported."),
                AstNode::Function(ref name, _, _) => {
                    if let AstNode::Name(mut name) = *name.clone() {
                        name = format!("{}.{}", root_block.prefix, name);
                        root_block.stable.insert(Identifier::new(name), SymbolData::new(SymbolType::Function));
                        backlog.push_front(stmt);
                    } else {
                        panic!("Function name must be a valid identifier.");
                    }
                }
                _ => panic!("Bad top level stmt. Should be let, function or module. is {:?}", stmt),
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

        let prefix = format!("{}.{}", self.prefix.clone(), name.as_str());
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
                        if let AstNode::Name(mut name) = current.pop_front().unwrap() {
                            name = format!("{}.{}", f.prefix, name);
                            f.stable.insert(Identifier::new(name), SymbolData::new(SymbolType::Variable));
                        } else {
                            panic!("Bad ast function params exprlist");
                        }
                        stack.push_front(list);
                    }
                    AstNode::Name(mut name) => {
                        name = format!("{}.{}", f.prefix, name);
                        f.stable.insert(Identifier::new(name), SymbolData::new(SymbolType::Variable));
                        if let AstNode::Name(mut name) = current.pop_front().unwrap() {
                            name = format!("{}.{}", f.prefix, name);
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
                let mut blocks = Self::parse_stmt_list(&f.prefix, &mut f.stable, list);
                for x in f.stable.keys() {
                    info!("{:?}", x);
                }
                f.children.append(&mut blocks);
            } else {
                panic!("body not statlist");
            }
        }

        self.children.push(f);
    }

    pub fn cat(a: String, b: String) -> String {
        format!("{}{}", a, b)
    }

    pub fn parse_stmt_list(prefix: &String, stable: &mut BTreeMap<Identifier, SymbolData>, list: VecDeque<AstNode>) -> Vec<Block> {
        let mut result: Vec<Block> = Vec::new();
        let mut current: Option<Block> = None;

        let push_stmts =
            |result: &mut Vec<Block>, mut stmts: VecDeque<Statement>, current: &mut Option<Block>, stable: &mut BTreeMap<Identifier, SymbolData>| {
                if let Some(ref mut unwrapped_current) = current {
                    // The result must be pushed outside the match
                    // because the borrow checker complains otherwise.
                    let should_push_to_result = match &mut unwrapped_current.block_type {
                        BlockType::Code(list) => {
                            list.append(&mut stmts);
                            false
                        }
                        _ => true,
                    };
                    if should_push_to_result {
                        result.push(current.take().unwrap());
                        *current = Some(Block::new(format!("{}.code{}", prefix, result.len()), BlockType::Code(stmts), stable.clone()));
                    }
                } else {
                    *current = Some(Block::new(format!("{}.code{}", prefix, result.len()), BlockType::Code(stmts), stable.clone()));
                };
            };

        for (i, stmt) in list.into_iter().enumerate() {
            match stmt {
                AstNode::Let(name, type_expr, val) => {
                    let mut final_stmts = VecDeque::new();
                    let let_stmts = Block::parse_let(name, type_expr, val);
                    for mut x in let_stmts {
                        x.ident.name = format!("{}.{}", prefix, x.ident.name);
                        stable.insert(x.ident.clone(), SymbolData::new(SymbolType::Variable));
                        final_stmts.push_back(Statement::Let(x));
                    }
                    push_stmts(&mut result, final_stmts, &mut current, stable);
                }
                AstNode::Assign(_name, _val) => {
                    // let let_stmts = Block::parse_let(name, None, Some(val);
                    // for mut x in let_stmts {
                    //     x.ident.name = format!("{}.{}", prefix, x.ident.name);
                    //     stable.insert(x.ident.clone(), SymbolData::new(SymbolType::Variable));
                    //     result.push_back(Statement::Let(x));
                    // }
                }
                AstNode::Return(val) => {
                    let mut final_stmts = VecDeque::new();
                    if let Some(val) = val {
                        let let_stmts = Block::parse_let(Box::from(AstNode::Name(format!("{}_return", i))), None, Some(val));
                        for mut x in let_stmts {
                            x.ident.name = format!("{}.{}", prefix, x.ident.name);
                            let return_val = x.ident.name.clone();
                            stable.insert(x.ident.clone(), SymbolData::new(SymbolType::Variable));
                            final_stmts.push_back(Statement::Let(x));
                            final_stmts.push_back(Statement::Return(Some(Value::Identifier(return_val))));
                        }
                    } else {
                        final_stmts.push_back(Statement::Return(None));
                    }
                    push_stmts(&mut result, final_stmts, &mut current, stable);
                }
                AstNode::If(condition, body, elseif) => {
                    // compute condtion and then add if block after the code block.
                    let mut final_stmts = VecDeque::new();
                    let cond_var = format!("{}_cond", i);
                    let let_stmts = Block::parse_let(Box::from(AstNode::Name(cond_var.clone())), None, Some(condition));
                    for mut x in let_stmts {
                        x.ident.name = format!("{}.{}", prefix, x.ident.name);
                        stable.insert(x.ident.clone(), SymbolData::new(SymbolType::Variable));
                        final_stmts.push_back(Statement::Let(x));
                    }
                    push_stmts(&mut result, final_stmts, &mut current, stable);
                    if let Some(b) = current.take() {
                        result.push(b);
                    }
                    let block = Self::parse_if(
                        format!("{}.if{}", prefix, result.len()),
                        stable.clone(),
                        Value::Identifier(cond_var),
                        body,
                        elseif,
                    );
                    result.push(block);
                }
                _ => panic!("Invalid statment"),
            }
        }

        // Get any stragglers in there
        if let Some(b) = current {
            result.push(b);
        }
        return result;
    }
    pub fn parse_if(
        prefix: String,
        mut stable: BTreeMap<Identifier, SymbolData>,
        condition: Value,
        body: Option<Box<AstNode>>,
        _elseif: Option<Box<AstNode>>,
    ) -> Block {
        let mut blocks: Vec<Block> = Vec::new();

        if let Some(body) = body {
            if let AstNode::StatList(list) = *body {
                blocks = Self::parse_stmt_list(&prefix, &mut stable, list);
            } else {
                panic!("if body not statlist");
            }
        }

        let mut result = Block::new(prefix, BlockType::If(condition), stable.clone());
        result.children = blocks;
        result
    }

    pub fn parse_let(name: Box<AstNode>, _type_expr: Option<TypeExpr>, val: Option<Box<AstNode>>) -> Vec<Let> {
        let mut result = Vec::new();
        if let AstNode::Name(name) = *name {
            if let Some(val) = val {
                let mut intermediate = Self::expr_to_ssa(name, *val);
                result.append(&mut intermediate);
            } else {
                result.push(Let {
                    ident: Identifier { name },
                    val: None,
                });
            }
        } else {
            panic!("Invalid identifier in let statement");
        }
        result
    }

    pub fn ast_node_to_value(node: AstNode) -> Option<Value> {
        match node {
            AstNode::Unary(_, _) => todo!(),
            AstNode::Number(num) => Some(Value::Number(num)),
            AstNode::String(s) => Some(Value::Identifier(s)),
            AstNode::Name(s) => Some(Value::Identifier(s)),
            AstNode::FunctionCall(_, _)
            | AstNode::Let(_, _, _)
            | AstNode::Return(_)
            | AstNode::Module(_)
            | AstNode::StatList(_)
            | AstNode::Function(_, _, _)
            | AstNode::If(_, _, _)
            | AstNode::ExprThen(_, _)
            | AstNode::ElseIf(_, _, _)
            | AstNode::Else(_)
            | AstNode::For(_, _, _)
            | AstNode::Binary(_, _, _)
            | AstNode::ExprList(_)
            | AstNode::Assign(_, _)
            | AstNode::While(_, _) => None,
        }
    }

    pub fn expr_to_ssa(result_identifier: String, root: AstNode) -> Vec<Let> {
        let mut stack: Vec<(String, AstNode)> = Vec::new();
        let mut result: Vec<Let> = Vec::new();
        stack.push((result_identifier.clone(), root));

        let is_leaf = |x: &AstNode| -> bool {
            match x {
                AstNode::Unary(_, _) | AstNode::Number(_) | AstNode::String(_) | AstNode::Name(_) | AstNode::FunctionCall(_, _) => true,
                AstNode::Let(_, _, _)
                | AstNode::Return(_)
                | AstNode::Module(_)
                | AstNode::StatList(_)
                | AstNode::Function(_, _, _)
                | AstNode::If(_, _, _)
                | AstNode::ExprThen(_, _)
                | AstNode::ElseIf(_, _, _)
                | AstNode::Else(_)
                | AstNode::For(_, _, _)
                | AstNode::Binary(_, _, _)
                | AstNode::ExprList(_)
                | AstNode::Assign(_, _)
                | AstNode::While(_, _) => false,
            }
        };

        let mut cnt = 0;
        let mut new_name = || {
            cnt += 1;
            format!("{}_{}", cnt, result_identifier)
        };
        while !stack.is_empty() {
            let (name, current) = stack.pop().unwrap();

            match current {
                AstNode::Binary(left, op, right) => {
                    let is_left_leaf = is_leaf(&left);
                    let is_right_leaf = is_leaf(&right);

                    if is_left_leaf && is_right_leaf {
                        let left = Self::ast_node_to_value(*left).unwrap();
                        let right = Self::ast_node_to_value(*right).unwrap();

                        result.push(Let::new(Identifier::new(name), Some(Expr::Binary(left, op, right))));
                    } else if !is_right_leaf && !is_right_leaf {
                        let left_name = new_name();
                        let right_name = new_name();

                        result.push(Let::new(
                            Identifier::new(name),
                            Some(Expr::Binary(Value::Identifier(left_name.clone()), op, Value::Identifier(right_name.clone()))),
                        ));
                        stack.push((left_name, *left));
                        stack.push((right_name, *right));
                    } else if !is_left_leaf {
                        let right = Self::ast_node_to_value(*right).unwrap();
                        let left_name = new_name();
                        result.push(Let::new(
                            Identifier::new(name),
                            Some(Expr::Binary(Value::Identifier(left_name.clone()), op, right)),
                        ));
                        stack.push((left_name, *left))
                    } else if !is_right_leaf {
                        todo!();
                    }
                }
                AstNode::Unary(op, node) => {
                    let node_is_leaf = is_leaf(&node);
                    if node_is_leaf {
                        let val = Self::ast_node_to_value(*node).unwrap();
                        result.push(Let::new(Identifier::new(name), Some(Expr::Unary(op, val))));
                    } else {
                        todo!();
                    }
                }
                AstNode::Number(num) => result.push(Let::new(Identifier::new(name), Some(Expr::Single(Value::Number(num))))),
                AstNode::String(_) => todo!(),
                AstNode::Name(name) => result.push(Let::new(Identifier::new(name.clone()), Some(Expr::Single(Value::Identifier(name))))),
                AstNode::ExprList(_) => todo!(),
                AstNode::Assign(_, _) => todo!(),
                AstNode::FunctionCall(_, _) => todo!(),
                AstNode::Let(_, _, _)
                | AstNode::Return(_)
                | AstNode::Module(_)
                | AstNode::StatList(_)
                | AstNode::Function(_, _, _)
                | AstNode::If(_, _, _)
                | AstNode::ExprThen(_, _)
                | AstNode::ElseIf(_, _, _)
                | AstNode::Else(_)
                | AstNode::For(_, _, _)
                | AstNode::While(_, _) => todo!(),
            }
        }
        result.reverse();
        result
    }

    pub fn parse_assign(_name: Box<AstNode>, _val: Option<Box<AstNode>>) {
        info!("parsing assing");
    }

    pub fn add_if(&mut self, _val: AstNode) {}
}
