use crate::parser::{Node, ParseTree};
use std::cmp::max;
use crate::grammar::OpGrammar;
use crate::lexer::json::JsonData;

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

pub struct JsonParseTree {

}
impl ParseTree<JsonData> for JsonParseTree {
    fn new(root: Node<JsonData>, g: OpGrammar) -> Self {
        Self {}
    }

    fn print(&self) {
    }
}
