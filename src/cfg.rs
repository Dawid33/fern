use crate::parser::fern::AstNode;

pub struct ControlFlowNode {}

pub struct ControlFlowGraph {}

// Convert to
impl ControlFlowGraph {
    pub fn from(ast: Box<AstNode>) -> Self {
        match ast {
            _ => (),
        }
        println!("Hello, World");
        return Self {};
    }
}
