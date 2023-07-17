use crate::parser::fern::AstNode;

pub enum ControlFlowNode {}

pub struct ControlFlowGraph {}

impl ControlFlowGraph {
    pub fn from(ast: Box<AstNode>) -> Self {
        println!("Hello, World");
        return Self {};
    }
}
