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

struct Module {}

impl Module {
    pub fn from(_: Box<AstNode>) -> Self {
        println!("Hello, World");
        return Self {};
    }
}
