use crate::grammar::OpGrammar;
use crate::parser::{Node, ParseTree};
use std::cmp::max;

#[derive(Debug, Clone)]
pub enum JsonValue {
    Null,
    String(String),
    Number(Number),
    Boolean(bool),
    Object(Object),
    Array(Vec<JsonValue>),
}

#[derive(Debug, Clone)]
pub struct Number {}

#[derive(Debug, Clone)]
pub struct Object {}

pub struct JsonParseTree {}
impl ParseTree for JsonParseTree {
    fn new(_root: Node, _g: OpGrammar) -> Self {
        Self {}
    }

    fn print(&self) {}
}
