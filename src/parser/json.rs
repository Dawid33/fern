use crate::parser::{Node, ParseTree};
use std::cmp::max;

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

impl Into<JsonValue> for ParseTree {
    fn into(self) -> JsonValue {
        // let mut node_stack: Vec<&Node> = vec![&self.root];
        // let mut child_count_stack: Vec<(i32, i32)> = vec![(0, (self.root.children.len() - 1) as i32)];
        //
        // while !node_stack.is_empty() {
        //     let current = node_stack.pop().unwrap();
        //     let (mut current_child, max_child) = child_count_stack.pop().unwrap();
        //
        //     while current.children.len() > 0 && current_child <= max_child {
        //         if !current
        //             .children
        //             .get(current_child as usize)
        //             .unwrap()
        //             .children
        //             .is_empty()
        //         {
        //             node_stack.push(current);
        //             let child = current.children.get(current_child as usize).unwrap();
        //             current_child -= 1;
        //             node_stack.push(child);
        //             child_count_stack.push((current_child, max_child));
        //             child_count_stack.push((0, (child.children.len() - 1) as i32));
        //             break;
        //         }
        //         current_child += 1;
        //     }
        // }

        JsonValue::Null
    }
}
