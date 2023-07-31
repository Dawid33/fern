use crate::parser::fern::AstNode;

pub struct ControlFlowNode {}

pub struct ControlFlowGraph {}

// Convert to
impl ControlFlowGraph {
    pub fn from(_: Box<AstNode>) -> Self {
        println!("Hello, World");
        return Self {};
    }
}
