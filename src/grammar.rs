use std::collections::hash_map::HashMap;

#[derive(Eq, PartialEq, Hash)]
pub enum Token {
    Start,
    Delim,
    A,
    B,
    Plus,
    Multiply,
    Number,
    Lparen,
    Rparen
}

pub struct Rule {
}

#[derive(Clone)]
enum Associativity {
    None,
    Left,
    Right,
    Equal,
    Undefined,
}

pub struct Grammar {
    non_terminals: Vec<Token>,
    terminals: Vec<Token>,
    delim: Token,
    axiom: Token,
    inverse_rewrite_rules: HashMap<Token, Vec<Token>>,
    rules: Vec<Rule>,
    op_table: HashMap<Token, HashMap<Token, Associativity>>,
}

impl Grammar {
    pub fn new(rules: Vec<Rule>, terminals: Vec<Token>, non_terminals: Vec<Token>, axiom: Token, delim: Token) -> Grammar {
        let result = Grammar {
            rules,
            terminals,
            non_terminals,
            axiom,
            delim,
            inverse_rewrite_rules: HashMap::new(),
            op_table: HashMap::new(),
        };

        return result;
    }
    pub fn get_precedence(&self, left: Token, right: Token) -> Associativity {
        return self.op_table.get(&left).unwrap().get(&right).unwrap().clone();
    }
}