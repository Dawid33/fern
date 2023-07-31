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
    root: Node<FernData>,
}

#[derive(Debug, Clone)]
pub enum TypeExpr {}

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

pub struct Binary {
    left: Box<AstNode>,
    op: Operator,
    right: Box<AstNode>,
}

pub struct Unary {
    op: Operator,
    val: Box<AstNode>,
}

pub struct ExprList {
    list: VecDeque<AstNode>,
}

pub struct Assign {
    identifier: Box<AstNode>,
    val: Box<AstNode>,
}

pub struct Let {
    identifier: Box<AstNode>,
    type_expr: Option<TypeExpr>,
    val: Option<Box<AstNode>>,
}

pub struct StatList {
    list: VecDeque<AstNode>,
}

pub struct Function {
    name: String,
    params: Option<Box<AstNode>>,
    body: Option<Box<AstNode>>,
}

pub struct If {
    expr: Box<AstNode>,
    body: Option<Box<AstNode>>,
    else_or_elseif: Option<Box<AstNode>>,
}

pub struct ExprThen {
    expr: Box<AstNode>,
    body: Option<Box<AstNode>>,
}

pub struct ElseIf {
    expr: Box<AstNode>,
    body: Option<Box<AstNode>>,
    else_or_else_if: Option<Box<AstNode>>,
}

pub enum AstNode {
    Binary(Binary),
    Unary(Unary),
    Number(i64),
    String(String),
    Name(String),
    ExprList(ExprList),
    Assign(Assign),
    Let(Let),
    Return(Option<Box<AstNode>>),
    Module(StatList),
    StatList(StatList),
    Function(Function),
    If(If),
    ExprThen(ExprThen),
    ElseIf(ElseIf),
    Else(Option<Box<AstNode>>),
    For(Box<AstNode>, Box<AstNode>, Box<AstNode>),
    While(Box<AstNode>, Box<AstNode>),
}
