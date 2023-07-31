use crate::grammar::{OpGrammar, Token};
use crate::lexer::fern::{FernData, FernTokens};
use crate::parser::fern::Operator::{
    Add, Divide, Equal, GreaterThan, GreaterThanOrEqual, LessThan, LessThanOrEqual, Modulo, Multiply, NotEqual,
    Subtract,
};
use crate::parser::{Node, ParseTree};
use log::info;
use simple_error::SimpleError;
use std::borrow::Cow;
use std::cmp::max;
use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::fmt::{Debug, Formatter};
use std::io::Write;
use std::os::unix::fs::symlink;
use std::sync;

pub struct FernParseTree {
    pub g: OpGrammar,
    pub root: Node<FernData>,
}

#[derive(Clone)]
pub enum Operator {
    Add,
    Multiply,
    Divide,
    Modulo,
    Subtract,
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
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
    ExprList(VecDeque<AstNode>),
    Assign(Box<AstNode>, Box<AstNode>),
    Let(Box<AstNode>, Option<TypeExpr>, Option<Box<AstNode>>),
    Return(Option<Box<AstNode>>),
    Module(Box<AstNode>),
    StatList(VecDeque<AstNode>),
    Function(Box<AstNode>, Option<Box<AstNode>>, Option<Box<AstNode>>),
    If(Box<AstNode>, Option<Box<AstNode>>, Option<Box<AstNode>>),
    ExprThen(Box<AstNode>, Option<Box<AstNode>>),
    ElseIf(Box<AstNode>, Option<Box<AstNode>>, Option<Box<AstNode>>),
    Else(Option<Box<AstNode>>),
    For(Box<AstNode>, Box<AstNode>, Box<AstNode>),
    While(Box<AstNode>, Box<AstNode>),
}

fn reduce<T: Debug>(
    node: Node<T>,
    stack: &mut Vec<VecDeque<AstNode>>,
    tok: &FernTokens,
    g: &OpGrammar,
) -> Option<AstNode> {
    if let Some(mut last) = stack.pop() {
        let (reduced, last) = if node.symbol == tok.base_exp {
            (Some(last.pop_front().unwrap()), Some(last))
        } else if node.symbol == tok.n_name {
            (Some(last.pop_front().unwrap()), Some(last))
        } else if node.symbol == tok.additive_exp {
            let right = last.pop_front().unwrap();
            let left = last.pop_front().unwrap();
            let result = if let Some(op) = node.children.get(0) {
                let result = if op.symbol == tok.plus {
                    (
                        Some(AstNode::Binary(Box::from(left), Add, Box::from(right))),
                        Some(last),
                    )
                } else if op.symbol == tok.minus {
                    (
                        Some(AstNode::Binary(Box::from(left), Subtract, Box::from(right))),
                        Some(last),
                    )
                } else {
                    panic!("Badly formed additive node in parse tree.");
                };
                result
            } else {
                panic!("Badly formed additive node in parse tree.");
            };
            result
        } else if node.symbol == tok.multiplicative_exp {
            let right = last.pop_front().unwrap();
            let left = last.pop_front().unwrap();
            let result = if let Some(op) = node.children.get(0) {
                let result = if op.symbol == tok.asterisk {
                    (
                        Some(AstNode::Binary(Box::from(left), Multiply, Box::from(right))),
                        Some(last),
                    )
                } else if op.symbol == tok.divide {
                    (
                        Some(AstNode::Binary(Box::from(left), Divide, Box::from(right))),
                        Some(last),
                    )
                } else if op.symbol == tok.percent {
                    (
                        Some(AstNode::Binary(Box::from(left), Modulo, Box::from(right))),
                        Some(last),
                    )
                } else {
                    panic!("Badly formed multiplicative node in parse tree.");
                };
                result
            } else {
                panic!("Badly formed multiplicative node in parse tree.");
            };
            result
        } else if node.symbol == tok.relational_exp {
            let right = last.pop_front().unwrap();
            let left = last.pop_front().unwrap();
            let result = if let Some(op) = node.children.get(0) {
                let result = if op.symbol == tok.lt {
                    (
                        Some(AstNode::Binary(Box::from(left), LessThan, Box::from(right))),
                        Some(last),
                    )
                } else if op.symbol == tok.gt {
                    (
                        Some(AstNode::Binary(Box::from(left), GreaterThan, Box::from(right))),
                        Some(last),
                    )
                } else if op.symbol == tok.lteq {
                    (
                        Some(AstNode::Binary(Box::from(left), LessThanOrEqual, Box::from(right))),
                        Some(last),
                    )
                } else if op.symbol == tok.gteq {
                    (
                        Some(AstNode::Binary(Box::from(left), GreaterThanOrEqual, Box::from(right))),
                        Some(last),
                    )
                } else if op.symbol == tok.neq {
                    (
                        Some(AstNode::Binary(Box::from(left), NotEqual, Box::from(right))),
                        Some(last),
                    )
                } else if op.symbol == tok.eq2 {
                    (
                        Some(AstNode::Binary(Box::from(left), Equal, Box::from(right))),
                        Some(last),
                    )
                } else {
                    panic!("Badly formed multiplicative node in parse tree.");
                };
                result
            } else {
                panic!("Badly formed multiplicative node in parse tree.");
            };
            result
        } else if node.symbol == tok.n_expr_then {
            let expr = last.pop_back().unwrap();
            let body = last.pop_back();
            let result = if let Some(b) = body {
                (Some(AstNode::ExprThen(Box::from(expr), Some(Box::from(b)))), None)
            } else {
                (Some(AstNode::ExprThen(Box::from(expr), None)), None)
            };
            result
        } else if node.symbol == tok.n_else_if_block {
            let result = if let Some(first) = node.children.first() {
                let result = if first.symbol == tok.else_t {
                    let result = if let Some(else_block) = last.pop_front() {
                        (Some(AstNode::Else(Some(Box::from(else_block)))), None)
                    } else {
                        (Some(AstNode::Else(None)), None)
                    };
                    result
                } else if first.symbol == tok.elseif {
                    let expr = Box::from(last.pop_back().unwrap());
                    let result = if let Some(first) = last.pop_back() {
                        match first {
                            AstNode::StatList(_) => {
                                let result = if let Some(second) = last.pop_back() {
                                    match second {
                                        AstNode::ElseIf(_, _, _) | AstNode::Else(_) => (
                                            Some(AstNode::ElseIf(
                                                expr,
                                                Some(Box::from(first)),
                                                Some(Box::from(second)),
                                            )),
                                            None,
                                        ),
                                        _ => {
                                            panic!("Badly formed else if / else statement.");
                                        }
                                    }
                                } else {
                                    (Some(AstNode::ElseIf(expr, Some(Box::from(first)), None)), None)
                                };
                                result
                            }
                            AstNode::ElseIf(_, _, _) => {
                                (Some(AstNode::ElseIf(expr, None, Some(Box::from(first)))), None)
                            }
                            _ => {
                                panic!("Badly formed else if / else statement.");
                            }
                        }
                    } else {
                        (Some(AstNode::ElseIf(expr, None, None)), None)
                    };
                    result
                } else {
                    panic!("Badly formed else if / else statement.");
                };
                result
            } else {
                panic!("Badly formed else if statement.");
            };
            result
        } else if node.symbol == tok.n_ret_stat {
            let exp = last.pop_front();
            let result = if let Some(exp) = exp {
                (Some(AstNode::Return(Some(Box::from(exp)))), None)
            } else {
                (Some(AstNode::Return(None)), None)
            };
            result
        } else if node.symbol == tok.n_stat {
            let result = if let Some(first) = node.children.first() {
                let result = if first.symbol == tok.let_t {
                    let exp = last.pop_front().unwrap();
                    let name = last.pop_front();
                    let result = if let Some(name) = name {
                        (Some(AstNode::Let(Box::from(name), None, Some(Box::from(exp)))), None)
                    } else {
                        (Some(AstNode::Let(Box::from(exp), None, None)), None)
                    };
                    result
                } else if first.symbol == tok.if_t {
                    let exprThen = last.pop_back().unwrap();
                    let result = if let AstNode::ExprThen(expr, body) = exprThen {
                        let result = if let Some(else_if_block) = last.pop_front() {
                            (Some(AstNode::If(expr, body, Some(Box::from(else_if_block)))), None)
                        } else {
                            (Some(AstNode::If(expr, body, None)), None)
                        };
                        result
                    } else {
                        panic!("Badly formed if statement.");
                    };
                    result
                } else if first.symbol == tok.fn_t {
                    let name = Box::from(last.pop_back().unwrap());
                    let result = if let Some(first) = last.pop_back() {
                        match first {
                            AstNode::ExprList(_) | AstNode::Name(_) => {
                                let result = if let Some(second) = last.pop_back() {
                                    match second {
                                        AstNode::If(_, _, _) | AstNode::Let(_, _, _) | AstNode::StatList(_) => (
                                            Some(AstNode::Function(
                                                name,
                                                Some(Box::from(first)),
                                                Some(Box::from(second)),
                                            )),
                                            None,
                                        ),
                                        _ => {
                                            panic!("Badly formed function definition.");
                                        }
                                    }
                                } else {
                                    (Some(AstNode::Function(name, Some(Box::from(first)), None)), None)
                                };
                                result
                            }
                            AstNode::If(_, _, _) | AstNode::Let(_, _, _) | AstNode::StatList(_) => {
                                (Some(AstNode::Function(name, None, Some(Box::from(first)))), None)
                            }
                            _ => {
                                panic!("Badly formed function definition.");
                            }
                        }
                    } else {
                        (Some(AstNode::Function(name, None, None)), None)
                    };
                    result
                } else if let Some(first) = last.pop_back() {
                    let result = match first {
                        AstNode::Name(_) => {
                            let expr = last.pop_front().unwrap();
                            (Some(AstNode::Assign(Box::from(first), Box::from(expr))), None)
                        }
                        AstNode::Return(_) => (Some(first), None),
                        _ => (None, None),
                    };
                    result
                } else {
                    panic!("Either a missing statement parse in ast gen or a bug. Actually its a bug either way.");
                };
                result
            } else {
                panic!("Either a missing statement parse in ast gen or a bug. Actually its a bug either way.");
            };

            result
        } else if node.symbol == tok.expr_list {
            let mut list = VecDeque::new();
            for x in last {
                if let AstNode::StatList(child_list) = x {
                    for x in child_list.into_iter().rev() {
                        list.push_front(x);
                    }
                } else {
                    list.push_front(x);
                }
            }
            (Some(AstNode::ExprList(list)), None)
        } else if node.symbol == tok.n_stat_list {
            let mut list = VecDeque::new();
            for x in last {
                if let AstNode::StatList(child_list) = x {
                    for x in child_list.into_iter().rev() {
                        list.push_front(x);
                    }
                } else {
                    list.push_front(x);
                }
            }
            (Some(AstNode::StatList(list)), None)
        } else {
            (None, None)
        };

        if let Some(parent) = stack.last_mut() {
            if let Some(reduced) = reduced {
                parent.push_back(reduced);
            }
        } else if let Some(reduced) = reduced {
            return Some(AstNode::StatList(VecDeque::from([reduced])));
        } else if let Some(last) = last {
            return Some(AstNode::StatList(last));
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

        let mut stack: Vec<VecDeque<AstNode>> = Vec::new();
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
                    b.push_str(
                        format!(
                            "{}",
                            self.g
                                .token_raw
                                .get(&current.children.get(current_child as usize).unwrap().symbol)
                                .unwrap()
                        )
                        .as_str(),
                    );
                    info!("{}", b);
                    b.clear();

                    // Go deeper or process current node.
                    if !current
                        .children
                        .get(current_child as usize)
                        .unwrap()
                        .children
                        .is_empty()
                    {
                        // Push onto stack
                        stack.push(VecDeque::new());

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
                        let wrong_data = || panic!("I'm too tired to write this error message properly.");
                        if let Some(last) = stack.last_mut() {
                            if let Some(data) = child.data {
                                match data {
                                    FernData::Number(n) => {
                                        if child.symbol == tok.number {
                                            last.push_back(AstNode::Number(n));
                                        } else {
                                            wrong_data();
                                        }
                                    }
                                    FernData::String(s) => {
                                        if child.symbol == tok.name {
                                            last.push_back(AstNode::Name(s));
                                        } else if child.symbol == tok.string {
                                            last.push_back(AstNode::String(s));
                                        } else {
                                            wrong_data();
                                        }
                                    }
                                    FernData::NoData => (),
                                }
                            }
                        }
                    }
                    current_child -= 1;
                    if current_child < min_child {
                        if let Some(root) = reduce(current, &mut stack, &tok, &self.g) {
                            return Ok(root);
                        }
                        break;
                    }
                }
            } else {
                if let Some(root) = reduce(current, &mut stack, &tok, &self.g) {
                    return Ok(root);
                }
            }
        }
        Err(SimpleError::new("Failed to build full ast from parse tree."))
    }
}
