use crate::parser::{Node, ParseTree};
use std::cmp::max;
use crate::lexer::json::JsonData;

#[derive(Debug, Clone)]
pub enum JsonValue {
    Null,
    // Short(Short),
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

// impl Into<JsonValue> for dyn ParseTree<JsonData> {
//     fn into(self) -> JsonValue {
//         JsonValue::Null
//     }
// }
