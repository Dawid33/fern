use crate::grammar::{OpGrammar, Token};
use crate::lexer::fern::{FernData, FernTokens};
use crate::parser::fern_ast::Operator::{Add, Divide, Equal, GreaterThan, GreaterThanOrEqual, LessThan, LessThanOrEqual, Modulo, Multiply, NotEqual, Subtract};
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
    FunctionCall(Box<AstNode>, Option<Box<AstNode>>),
    Function(Box<AstNode>, Option<Box<AstNode>>, Option<Box<AstNode>>),
    If(Box<AstNode>, Option<Box<AstNode>>, Option<Box<AstNode>>),
    ExprThen(Box<AstNode>, Option<Box<AstNode>>),
    ElseIf(Box<AstNode>, Option<Box<AstNode>>, Option<Box<AstNode>>),
    Else(Option<Box<AstNode>>),
    For(Box<AstNode>, Box<AstNode>, Box<AstNode>),
    While(Box<AstNode>, Box<AstNode>),
}

/// Reduce a node of the parse tree into an ast node.
fn new_reduce<T: Debug>(node: Node<T>, stack: &mut Vec<VecDeque<AstNode>>, tok: &FernTokens, g: &OpGrammar) -> Option<AstNode> {
    let mut last = if let Some(last) = stack.pop() {
        last
    } else {
        panic!("Cannot reduce an empty stack. Probably finished traversing parse tree too early.");
    };

    let reduced: Result<AstNode, SimpleError>;
    if node.symbol == tok.base_exp {
        reduced = Ok(last.pop_front().unwrap());
    } else if node.symbol == tok.n_name {
        reduced = Ok(last.pop_front().unwrap());
    } else if node.symbol == tok.additive_exp {
        reduced = reduce_additive_exp(node, last, tok);
    } else if node.symbol == tok.multiplicative_exp {
        reduced = reduce_multiplicative_exp(node, last, tok);
    } else if node.symbol == tok.relational_exp {
        reduced = reduce_relational_exp(node, last, tok);
    } else if node.symbol == tok.n_stat {
        reduced = reduce_stat(node, last, tok);
    } else if node.symbol == tok.n_else_if_block {
        reduced = reduce_else_if(node, last, tok);
    } else if node.symbol == tok.n_function_call {
        let expr = last.pop_back().unwrap();
        let body = last.pop_back();
        let result = if let Some(b) = body {
            Ok(AstNode::FunctionCall(Box::from(expr), Some(Box::from(b))))
        } else {
            Ok(AstNode::FunctionCall(Box::from(expr), None))
        };
        reduced = result
    } else if node.symbol == tok.n_expr_then {
        let expr = last.pop_back().unwrap();
        let body = last.pop_back();
        let result = if let Some(b) = body {
            Ok(AstNode::ExprThen(Box::from(expr), Some(Box::from(b))))
        } else {
            Ok(AstNode::ExprThen(Box::from(expr), None))
        };
        reduced = result
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
        reduced = Ok(AstNode::StatList(list))
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
        reduced = Ok(AstNode::ExprList(list))
    } else if node.symbol == tok.n_ret_stat {
        let exp = last.pop_front();
        reduced = if let Some(exp) = exp {
            Ok(AstNode::Return(Some(Box::from(exp))))
        } else {
            Ok(AstNode::Return(None))
        };
    } else {
        panic!(
            "Parse tree node not recognized = {:?}. Probably changed grammar and didn't update ast transform you bad boy.",
            g.token_raw.get(&node.symbol).unwrap()
        );
    }

    if let Some(parent) = stack.last_mut() {
        if let Ok(reduced) = reduced {
            parent.push_back(reduced);
        }
    } else if let Ok(reduced) = reduced {
        // return Some(AstNode::Module(VecDeque::from([reduced])));
        if let AstNode::StatList(_) = reduced {
            return Some(AstNode::Module(Box::from(reduced)));
        } else {
            return Some(AstNode::Module(Box::from(AstNode::StatList(VecDeque::from_iter([reduced].into_iter())))));
        }
    } else {
        panic!("Cannot reduce, fix buggo.")
    }
    None
}

fn reduce_additive_exp<T: Debug>(node: Node<T>, mut last: VecDeque<AstNode>, tok: &FernTokens) -> Result<AstNode, SimpleError> {
    let right = last.pop_front().unwrap();
    let left = last.pop_front().unwrap();
    let result = if let Some(op) = node.children.get(0) {
        let result = if op.symbol == tok.plus {
            Ok(AstNode::Binary(Box::from(left), Add, Box::from(right)))
        } else if op.symbol == tok.minus {
            Ok(AstNode::Binary(Box::from(left), Subtract, Box::from(right)))
        } else {
            Err(SimpleError::new("Badly formed additive node in parse tree."))
        };
        result
    } else {
        Err(SimpleError::new("Badly formed additive node in parse tree."))
    };
    result
}

fn reduce_multiplicative_exp<T: Debug>(node: Node<T>, mut last: VecDeque<AstNode>, tok: &FernTokens) -> Result<AstNode, SimpleError> {
    let right = last.pop_front().unwrap();
    let left = last.pop_front().unwrap();
    let result = if let Some(op) = node.children.get(0) {
        let result = if op.symbol == tok.asterisk {
            Ok(AstNode::Binary(Box::from(left), Multiply, Box::from(right)))
        } else if op.symbol == tok.divide {
            Ok(AstNode::Binary(Box::from(left), Divide, Box::from(right)))
        } else if op.symbol == tok.percent {
            Ok(AstNode::Binary(Box::from(left), Modulo, Box::from(right)))
        } else {
            Err(SimpleError::new("Badly formed multiplicative node in parse tree."))
        };
        result
    } else {
        Err(SimpleError::new("Badly formed multiplicative node in parse tree."))
    };
    result
}

fn reduce_relational_exp<T: Debug>(node: Node<T>, mut last: VecDeque<AstNode>, tok: &FernTokens) -> Result<AstNode, SimpleError> {
    let right = last.pop_front().unwrap();
    let left = last.pop_front().unwrap();
    let result = if let Some(op) = node.children.get(0) {
        let result = if op.symbol == tok.lt {
            Ok(AstNode::Binary(Box::from(left), LessThan, Box::from(right)))
        } else if op.symbol == tok.gt {
            Ok(AstNode::Binary(Box::from(left), GreaterThan, Box::from(right)))
        } else if op.symbol == tok.lteq {
            Ok(AstNode::Binary(Box::from(left), LessThanOrEqual, Box::from(right)))
        } else if op.symbol == tok.gteq {
            Ok(AstNode::Binary(Box::from(left), GreaterThanOrEqual, Box::from(right)))
        } else if op.symbol == tok.neq {
            Ok(AstNode::Binary(Box::from(left), NotEqual, Box::from(right)))
        } else if op.symbol == tok.eq2 {
            Ok(AstNode::Binary(Box::from(left), Equal, Box::from(right)))
        } else {
            Err(SimpleError::new("Badly formed multiplicative node in parse tree."))
        };
        result
    } else {
        Err(SimpleError::new("Badly formed multiplicative node in parse tree."))
    };
    result
}

fn reduce_stat<T: Debug>(node: Node<T>, mut last: VecDeque<AstNode>, tok: &FernTokens) -> Result<AstNode, SimpleError> {
    let result = if let Some(first) = node.children.first() {
        let result = if first.symbol == tok.let_t {
            let exp = last.pop_front().unwrap();
            let name = last.pop_front();
            let result = if let Some(name) = name {
                Ok(AstNode::Let(Box::from(name), None, Some(Box::from(exp))))
            } else {
                Ok(AstNode::Let(Box::from(exp), None, None))
            };
            result
        } else if first.symbol == tok.if_t {
            reduce_if(last)
        } else if first.symbol == tok.fn_t {
            function(node, last, tok)
        } else if let Some(first) = last.pop_back() {
            let result = match first {
                AstNode::Name(_) => {
                    let expr = last.pop_front().unwrap();
                    Ok(AstNode::Assign(Box::from(first), Box::from(expr)))
                }
                AstNode::Return(_) => Ok(first),
                _ => Err(SimpleError::new("Unkown statement in statement list.")),
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
}

fn function<T: Debug>(_: Node<T>, mut last: VecDeque<AstNode>, _: &FernTokens) -> Result<AstNode, SimpleError> {
    let name = Box::from(last.pop_back().unwrap());
    let result = if let Some(first) = last.pop_back() {
        match first {
            AstNode::ExprList(_) | AstNode::Name(_) => {
                let result = if let Some(second) = last.pop_back() {
                    match second {
                        AstNode::If(_, _, _) | AstNode::Let(_, _, _) | AstNode::StatList(_) => {
                            Ok(AstNode::Function(name, Some(Box::from(first)), Some(Box::from(second))))
                        }
                        _ => Err(SimpleError::new("Badly formed function definition.")),
                    }
                } else {
                    Ok(AstNode::Function(name, Some(Box::from(first)), None))
                };
                result
            }
            AstNode::If(_, _, _) | AstNode::Let(_, _, _) | AstNode::StatList(_) => Ok(AstNode::Function(name, None, Some(Box::from(first)))),
            _ => Err(SimpleError::new("Badly formed function definition.")),
        }
    } else {
        Ok(AstNode::Function(name, None, None))
    };
    result
}

fn reduce_if(mut last: VecDeque<AstNode>) -> Result<AstNode, SimpleError> {
    let expr_then = last.pop_back().unwrap();
    let result = if let AstNode::ExprThen(expr, body) = expr_then {
        let result = if let Some(else_if_block) = last.pop_front() {
            Ok(AstNode::If(expr, body, Some(Box::from(else_if_block))))
        } else {
            Ok(AstNode::If(expr, body, None))
        };
        result
    } else {
        Err(SimpleError::new("Badly formed if statement."))
    };
    result
}

fn reduce_else_if<T: Debug>(node: Node<T>, mut last: VecDeque<AstNode>, tok: &FernTokens) -> Result<AstNode, SimpleError> {
    let result = if let Some(first) = node.children.first() {
        let result = if first.symbol == tok.else_t {
            let result = if let Some(else_block) = last.pop_front() {
                Ok(AstNode::Else(Some(Box::from(else_block))))
            } else {
                Ok(AstNode::Else(None))
            };
            result
        } else if first.symbol == tok.elseif {
            let expr = Box::from(last.pop_back().unwrap());
            let result = if let Some(first) = last.pop_back() {
                match first {
                    AstNode::StatList(_) => {
                        let result = if let Some(second) = last.pop_back() {
                            match second {
                                AstNode::ElseIf(_, _, _) | AstNode::Else(_) => Ok(AstNode::ElseIf(expr, Some(Box::from(first)), Some(Box::from(second)))),
                                _ => {
                                    panic!("Badly formed else if / else statement.");
                                }
                            }
                        } else {
                            Ok(AstNode::ElseIf(expr, Some(Box::from(first)), None))
                        };
                        result
                    }
                    AstNode::ElseIf(_, _, _) => Ok(AstNode::ElseIf(expr, None, Some(Box::from(first)))),
                    _ => {
                        panic!("Badly formed else if / else statement.");
                    }
                }
            } else {
                Ok(AstNode::ElseIf(expr, None, None))
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
                    for _i in 0..child_count_stack.len() {
                        b.push_str("  ");
                    }
                    b.push_str(
                        format!(
                            "{}",
                            self.g.token_raw.get(&current.children.get(current_child as usize).unwrap().symbol).unwrap()
                        )
                        .as_str(),
                    );
                    info!("{}", b);
                    b.clear();

                    // Go deeper or process current node.
                    if !current.children.get(current_child as usize).unwrap().children.is_empty() {
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
                        if let Some(root) = new_reduce(current, &mut stack, &tok, &self.g) {
                            return Ok(root);
                            // return Ok(AstNode::Module(Box::from(AstNode::StatList(VecDeque::from_iter([root].into_iter())))));
                        }
                        break;
                    }
                }
            } else {
                if let Some(root) = new_reduce(current, &mut stack, &tok, &self.g) {
                    return Ok(root);
                    // return Ok(AstNode::Module(Box::from(AstNode::StatList(VecDeque::from_iter([root].into_iter())))));
                }
            }
        }
        Err(SimpleError::new("Failed to build full ast from parse tree."))
    }
}

// Possible alternative code, might try to refactor into this later maybe.

// pub struct FernParseTree {
//     pub g: OpGrammar,
//     root: Node<FernData>,
// }

// #[derive(Debug, Clone)]
// pub enum TypeExpr {}

// #[derive(Clone)]
// pub enum Operator {
//     Add,
//     Multiply,
//     Divide,
//     Modulo,
//     Subtract,
//     Equal,
//     NotEqual,
//     GreaterThan,
//     GreaterThanOrEqual,
//     LessThan,
//     LessThanOrEqual,
// }

// pub struct Binary {
//     left: Box<AstNode>,
//     op: Operator,
//     right: Box<AstNode>,
// }

// pub struct Unary {
//     op: Operator,
//     val: Box<AstNode>,
// }

// pub struct ExprList {
//     list: VecDeque<AstNode>,
// }

// pub struct Assign {
//     identifier: Box<AstNode>,
//     val: Box<AstNode>,
// }

// pub struct Let {
//     identifier: Box<AstNode>,
//     type_expr: Option<TypeExpr>,
//     val: Option<Box<AstNode>>,
// }

// pub struct StatList {
//     list: VecDeque<AstNode>,
// }

// pub struct Function {
//     name: String,
//     params: Option<Box<AstNode>>,
//     body: Option<Box<AstNode>>,
// }

// pub struct If {
//     expr: Box<AstNode>,
//     body: Option<Box<AstNode>>,
//     else_or_elseif: Option<Box<AstNode>>,
// }

// pub struct ExprThen {
//     expr: Box<AstNode>,
//     body: Option<Box<AstNode>>,
// }

// pub struct ElseIf {
//     expr: Box<AstNode>,
//     body: Option<Box<AstNode>>,
//     else_or_else_if: Option<Box<AstNode>>,
// }

// pub enum AstNode {
//     Binary(Binary),
//     Unary(Unary),
//     Number(i64),
//     String(String),
//     Name(String),
//     ExprList(ExprList),
//     Assign(Assign),
//     Let(Let),
//     Return(Option<Box<AstNode>>),
//     Module(StatList),
//     StatList(StatList),
//     Function(Function),
//     If(If),
//     ExprThen(ExprThen),
//     ElseIf(ElseIf),
//     Else(Option<Box<AstNode>>),
//     For(Box<AstNode>, Box<AstNode>, Box<AstNode>),
//     While(Box<AstNode>, Box<AstNode>),
// }
