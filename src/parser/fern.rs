use crate::parser::{Node, ParseTree};
use std::cmp::max;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::os::unix::fs::symlink;
use std::sync;
use log::info;
use crate::grammar::Token;
use crate::lexer::fern::{FernData, FernTokens};
use crate::parser::fern::Operator::{Add, Divide, GreaterThan, LessThan, Multiply};
use simple_error::SimpleError;
use crate::FernParseTree;
use crate::parser::fern::AstNode::{Else, ElseIf};

#[derive(Clone)]
pub enum Operator {
    Add,
    Multiply,
    Divide,
    Subtract,
    GreaterThan,
    LessThan,
}

#[derive(Debug, Clone)]
pub enum TypeExpr {}

#[derive(Clone)]
pub enum AstNode {
    Binary(Box<AstNode>, Operator, Box<AstNode>),
    Unary(Operator, Box<AstNode>),
    Number(i64),
    String(String),
    Name(String),
    NameList(Vec<AstNode>),
    Assign(Box<AstNode>, Box<AstNode>),
    Let(Box<AstNode>, Option<TypeExpr>, Box<AstNode>),
    Return(Option<Box<AstNode>>),
    Module(Vec<AstNode>),
    Function(Box<AstNode>, Box<AstNode>, Vec<AstNode>),
    If(Box<AstNode>, Vec<AstNode>, Option<Box<AstNode>>),
    ElseIf(Box<AstNode>, Vec<AstNode>, Option<Box<AstNode>>),
    Else(Vec<AstNode>),
    For(Box<AstNode>, Box<AstNode>, Vec<AstNode>),
    While(Box<AstNode>, Vec<AstNode>),
}
impl Debug for Operator{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Add => write!(f, "+"),
            Multiply => write!(f, "*"),
            Operator::Divide => write!(f, "/"),
            Operator::Subtract => write!(f, "-"),
            GreaterThan => write!(f, ">"),
            LessThan => write!(f, "<"),
        }
    }
}

impl Debug for AstNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AstNode::Binary(l, o, r) => write!(f, "{:?}", o),
            AstNode::Unary(o, e) => write!(f, "{:?}", o),
            AstNode::Number(n) => write!(f, "{}", n),
            AstNode::String(s) => { write!(f, "\"{}\"", s) }
            AstNode::Name(n) => write!(f, "{}", n),
            AstNode::NameList(_) => write!(f, "Name List"),
            AstNode::Assign(_, _) => write!(f, "="),
            AstNode::Let(_, _, _) => write!(f, "Let"),
            AstNode::Module(_) => write!(f, "Module"),
            AstNode::Function(name, _, _) => write!(f, "{:?}", name),
            AstNode::If(_, _, _) => write!(f, "If"),
            AstNode::ElseIf(_, _, _) => write!(f, "Else If"),
            AstNode::Else(_) => write!(f, "Else"),
            AstNode::For(_, _, _) => write!(f, "For"),
            AstNode::While(_, _) => write!(f, "While"),
            AstNode::Return(_) => write!(f, "Return"),
        }
    }
}

fn reduce<T: Debug>(node: Node<T>, stack: &mut Vec<Vec<AstNode>>, tok: &FernTokens) -> Option<AstNode> {
    if let Some(mut last) = stack.pop() {
        let (reduced, last) = if tok.asterisk == node.symbol {
            let left = last.pop().unwrap();
            let right = last.pop().unwrap();
            (Some(AstNode::Binary(Box::from(left), Multiply, Box::from(right))), Some(last))
        } else if tok.plus == node.symbol {
            let left = last.pop().unwrap();
            let right = last.pop().unwrap();
            (Some(AstNode::Binary(Box::from(left), Add, Box::from(right))), Some(last))
        } else if tok.divide == node.symbol {
            let left = last.pop().unwrap();
            let right = last.pop().unwrap();
            (Some(AstNode::Binary(Box::from(left), Divide, Box::from(right))), Some(last))
        } else if tok.gt == node.symbol {
            let left = last.pop().unwrap();
            let right = last.pop().unwrap();
            (Some(AstNode::Binary(Box::from(left), GreaterThan, Box::from(right))), Some(last))
        } else if tok.lt == node.symbol {
            let left = last.pop().unwrap();
            let right = last.pop().unwrap();
            (Some(AstNode::Binary(Box::from(left), LessThan, Box::from(right))), Some(last))
        } else if tok.return_t == node.symbol {
            if let Some(expr) = last.pop() {
                (Some(AstNode::Return(Some(Box::from(expr)))), Some(last))
            } else {
                (Some(AstNode::Return(None)), Some(last))
            }
        } else if tok.eq == node.symbol {
            let left = last.pop().unwrap();
            let right = last.pop().unwrap();
            (Some(AstNode::Assign(Box::from(left), Box::from(right))), Some(last))
        } else if tok.let_t == node.symbol {
            let eq = last.pop().unwrap();
            match eq {
                AstNode::Assign(s, expr) => (Some(AstNode::Let(s, None, expr)), Some(last)),
                _ => panic!("Invalid let statement. If you see this then you've probably found a lexer / parser bug."),
            }
        } else if tok.comma == node.symbol {
            (Some(AstNode::NameList(last.clone())), Some(last))
        } else if tok.if_t == node.symbol {
            let expr = last.pop().unwrap();
            let else_node = if let Some(first_of_last) = last.first() {
                match first_of_last {
                    Else(_) | ElseIf(_,_,_) => Some(Box::from(last.remove(0))),
                    _ => None
                }
            } else {
                None
            };
            (Some(AstNode::If(Box::from(expr), last.clone(), else_node)), Some(last))
        } else if tok.elseif == node.symbol {
            let expr = last.pop().unwrap();
            let else_node = if let Some(first_of_last) = last.first() {
                if let Else(_) = first_of_last {
                    Some(Box::from(last.remove(0)))
                } else {
                    None
                }
            } else {
                None
            };
            (Some(AstNode::ElseIf(Box::from(expr), last.clone(), else_node)), Some(last))
        } else if tok.else_t == node.symbol {
            (Some(AstNode::Else((last.clone()))), Some(last))
        } else if tok.while_t == node.symbol {
            let expr = last.pop().unwrap();
            (Some(AstNode::While(Box::from(expr), last.clone())), Some(last))
        } else if tok.for_t == node.symbol {
            let expr = last.pop().unwrap();
            let list = last.pop().unwrap();
            (Some(AstNode::For(Box::from(expr),Box::from(list), last.clone())), Some(last))
        } else if tok.fn_t == node.symbol {
            let name = last.pop().unwrap();
            let params = last.pop().unwrap();
            (Some(AstNode::Function(Box::from(name), Box::from(params), last.clone())), Some(last))
        } else if tok.semi == node.symbol {
            return if let Some(parent) = stack.last_mut() {
                for x in last {
                    parent.push(x);
                }
                None
            } else {
                Some(AstNode::Module(last))
            }
        } else {
            (None, None)
        };

        if let Some(parent) = stack.last_mut() {
            if let Some(reduced) = reduced {
                parent.push(reduced);
            }
        } else if let Some(reduced) = reduced {
            return Some(AstNode::Module(vec![reduced]));
        } else if let Some(last) = last {
            return Some(AstNode::Module(last));
        } else {
            panic!("Cannot reduce, fix buggo.")
        }
    } else {
        panic!("Cannot reduce an empty stack. Probably finished traversing parse tree too early.");
    }
    None
}

impl FernParseTree {
    pub fn build_ast(self) -> Result<AstNode, SimpleError> {
        let tok = FernTokens::new(&self.g.token_reverse);

        let mut stack: Vec<Vec<AstNode>> = Vec::new();
        let mut b = String::new();
        b.push_str(format!("{}", self.g.token_raw.get(&self.root.symbol).unwrap()).as_str());
        info!("{}", b);
        b.clear();

        let mut child_count_stack: Vec<(i32, i32)> = vec![((self.root.children.len() - 1) as i32, 0)];
        let mut node_stack: Vec<Node<FernData>> = vec![self.root];

        while !node_stack.is_empty() {
            let mut current = node_stack.pop().unwrap();
            let (mut current_child, min_child) = child_count_stack.pop().unwrap();

            if current.children.len() > 0 && current_child >= min_child {
                while current.children.len() > 0 && current_child >= min_child {
                    for i in 0..child_count_stack.len() {
                        b.push_str("  ");
                    }
                    b.push_str(format!("{}", self.g.token_raw.get(&current.children.get(current_child as usize).unwrap().symbol).unwrap()).as_str());
                    info!("{}", b);
                    b.clear();


                    // Go deeper or process current node.
                    if !current.children.get(current_child as usize).unwrap().children.is_empty() {
                        // Push onto stack
                        stack.push(vec![]);

                        let child = current.children.remove(current_child as usize);
                        current_child -= 1;
                        let len = (child.children.len() - 1) as i32;
                        node_stack.push(current);
                        node_stack.push(child);
                        child_count_stack.push((current_child, min_child));
                        child_count_stack.push((len, 0));
                        break;
                    } else {
                        let child = current.children.get(current_child as usize).unwrap().clone();
                        let wrong_data = || { panic!("I'm too tired to write this error message properly.") };
                        if let Some(last) = stack.last_mut() {
                            if let Some(data) = child.data {
                                match data {
                                    FernData::Number(n) => {
                                        if child.symbol == tok.number {
                                            last.push(AstNode::Number(n));
                                        } else  {
                                            wrong_data();
                                        }
                                    }
                                    FernData::String(s) => {
                                        if child.symbol == tok.name {
                                            last.push(AstNode::Name(s));
                                        } else if child.symbol == tok.string {
                                            last.push(AstNode::String(s));
                                        } else {
                                            wrong_data();
                                        }
                                    }
                                    FernData::NoData =>  ()
                                }
                            }
                        }
                    }
                    current_child -= 1;
                    if current_child < min_child {
                        if let Some(root) = reduce(current, &mut stack, &tok) {
                            return Ok(root);
                        }
                        break;
                    }
                }
            } else {
                if let Some(root) = reduce(current, &mut stack, &tok) {
                    return Ok(root);
                }
            }
        }
        Err(SimpleError::new("Failed to build full ast from parse tree."))
    }
}

