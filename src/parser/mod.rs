pub mod json;

use crate::grammar::{Associativity, Grammar, Rule};
use log::{debug, trace};
use std::any::Any;
use std::collections::{HashMap, LinkedList, VecDeque};
use std::error::Error;
use std::io::ErrorKind::AlreadyExists;
use std::io::Read;
use std::panic::{resume_unwind, set_hook};
use std::thread::current;

#[allow(unused)]
pub struct ParseTree {
    pub g: Grammar,
    root: Node,
}

impl ParseTree {
    #[allow(unused)]
    pub fn new(root: Node, g: Grammar) -> Self {
        Self { g, root }
    }
}

pub struct Node {
    symbol: u8,
    data: Option<String>,
    children: Vec<Node>,
}

impl Node {
    pub fn new(symbol: u8, data: Option<String>) -> Self {
        Self {
            symbol,
            data,
            children: Vec::new(),
        }
    }
    pub fn append_child(&mut self, other: Node) {
        self.children.push(other);
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
struct TokenGrammarTuple {
    token: u8,
    id: u64,
    associativity: Associativity,
}

impl TokenGrammarTuple {
    pub fn new(token: u8, associativity: Associativity, parser: &mut ParallelParser) -> Self {
        Self {
            token,
            associativity,
            id: parser.gen_id(),
        }
    }
}

impl ParseTree {
    pub fn print(&self) {
        let mut node_stack: Vec<&Node> = vec![&self.root];
        let mut child_count_stack: Vec<i32> = vec![(self.root.children.len() - 1) as i32];

        println!("{}", self.g.token_raw.get(&self.root.symbol).unwrap());
        while !node_stack.is_empty() {
            let current = node_stack.pop().unwrap();
            let mut current_child = child_count_stack.pop().unwrap();

            while current.children.len() > 0 && current_child >= 0 {
                for i in 0..child_count_stack.len() {
                    if *child_count_stack.get(i).unwrap() >= 0 {
                        print!("| ");
                    } else {
                        print!("  ");
                    }
                }
                if current_child != 0 {
                    println!(
                        "├─{}",
                        self.g
                            .token_raw
                            .get(&current.children.get(current_child as usize).unwrap().symbol)
                            .unwrap()
                    )
                } else {
                    println!(
                        "└─{}",
                        self.g
                            .token_raw
                            .get(&current.children.get(current_child as usize).unwrap().symbol)
                            .unwrap()
                    )
                }
                if !current
                    .children
                    .get(current_child as usize)
                    .unwrap()
                    .children
                    .is_empty()
                {
                    node_stack.push(current);
                    let child = current.children.get(current_child as usize).unwrap();
                    current_child -= 1;
                    node_stack.push(child);
                    child_count_stack.push(current_child);
                    child_count_stack.push((child.children.len() - 1) as i32);
                    break;
                }
                current_child -= 1;
            }
        }
    }
}

pub struct ParallelParser {
    stack: Vec<TokenGrammarTuple>,
    pub grammar: Grammar,
    open_nodes: HashMap<u64, Node>,
    should_reconsume: bool,
    highest_id: u64,
    iteration: u64,
}

impl ParallelParser {
    pub fn new(grammar: Grammar, threads: usize) -> Self {
        let _ = threads;
        let parser = Self {
            stack: Vec::new(),
            grammar,
            should_reconsume: false,
            open_nodes: HashMap::new(),
            highest_id: 0,
            iteration: 0,
        };

        return parser;
    }

    pub fn parse(&mut self, tokens: LinkedList<Vec<u8>>) {
        for t in tokens {
            for t in t {
                self.consume_token(&t).expect("Parser raised an exception.");
            }
        }
    }

    pub fn gen_id(&mut self) -> u64 {
        self.highest_id += 1;
        return self.highest_id;
    }

    fn consume_token(&mut self, token: &u8) -> Result<(), Box<dyn Error>> {
        if self.stack.is_empty() {
            let t = TokenGrammarTuple::new(*token, Associativity::Left, self);
            self.stack.push(t);
            return Ok(());
        }

        loop {
            self.iteration += 1;
            self.should_reconsume = false;

            if self.stack.len() == 1 {
                if self.stack.get(0).unwrap().token == self.grammar.axiom {
                    let mut output = String::new();
                    for (id, _) in &self.open_nodes {
                        for t in &self.stack {
                            if t.id == *id {
                                output.push_str(format!("{:?} ", t.token).as_str());
                                break;
                            }
                        }
                    }
                    debug!("{} FINAL Open nodes: {}", self.iteration, output);
                    self.print_stack();
                    return Ok(());
                }
            }

            let mut y: Option<TokenGrammarTuple> = None;
            for element in &self.stack {
                if self.grammar.terminals.contains(&element.token) {
                    y = Some(*element);
                }
            }

            if let None = y {
                let t = TokenGrammarTuple::new(*token, Associativity::Left, self);
                self.stack.push(t);
                return Ok(());
            }
            let y = y.unwrap();

            let precedence = if *token == self.grammar.delim {
                Associativity::Right
            } else {
                self.grammar.get_precedence(y.token, *token)
            };

            if precedence == Associativity::None {
                panic!(
                    "No precedence between y = {} and token = {} == user grammar error",
                    self.grammar.token_raw.get(&y.token).unwrap(),
                    self.grammar.token_raw.get(&token).unwrap()
                )
            }

            let mut output = String::new();
            for (key, node) in &self.open_nodes {
                output.push_str(
                    format!(
                        "({:?} {:?}) ",
                        key,
                        self.grammar.token_raw.get(&node.symbol).unwrap()
                    )
                    .as_str(),
                );
            }
            debug!("{} Open nodes: {}", self.iteration, output);
            debug!("{} Applying {:?} {:?}", self.iteration, token, precedence);
            self.print_stack();

            if precedence == Associativity::Left {
                let t = TokenGrammarTuple::new(*token, Associativity::Left, self);
                self.stack.push(t);
                debug!("{} Append", self.iteration);
                return Ok(());
            }

            if precedence == Associativity::Equal {
                let t = TokenGrammarTuple::new(*token, Associativity::Equal, self);
                self.stack.push(t);
                debug!("{} Append", self.iteration);
                return Ok(());
            }

            if self.grammar.non_terminals.contains(token) {
                let t = TokenGrammarTuple::new(*token, Associativity::Undefined, self);
                self.stack.push(t);
                debug!("{}, Append", self.iteration);
                return Ok(());
            }

            if precedence == Associativity::Right {
                let mut i: i32 = -1;
                for (j, x) in self.stack.iter().enumerate() {
                    if x.associativity == Associativity::Left {
                        i = j as i32;
                    }
                }

                if i < 0 {
                    let t = TokenGrammarTuple::new(*token, Associativity::Right, self);
                    self.stack.push(t);
                    debug!("{}, Append", self.iteration);
                    return Ok(());
                } else {
                    if i - 1 >= 0 {
                        let xi_minus_one = self.stack.get((i - 1) as usize).unwrap();

                        if self.grammar.terminals.contains(&xi_minus_one.token) {
                            self.process_terminal(i);
                        } else if self.grammar.non_terminals.contains(&xi_minus_one.token) {
                            self.process_non_terminal(i);
                        } else {
                            panic!("Should be able to reduce but cannot.");
                        }
                    } else {
                        self.process_terminal(i);
                    }
                }
            }
            if !self.should_reconsume {
                break;
            }
        }
        Ok(())
    }

    fn process_terminal(&mut self, i: i32) {
        self.reduce_stack(i, 0);
    }

    fn process_non_terminal(&mut self, i: i32) {
        self.reduce_stack(i, -1);
    }

    fn reduce_stack(&mut self, i: i32, offset: i32) {
        let mut rule: Option<Rule> = None;
        let mut apply_rewrites: HashMap<u8, u8> = HashMap::new();
        let mut longest: i32 = 0;

        for r in &self.grammar.rules {
            let mut rewrites: HashMap<u8, u8> = HashMap::new();
            let mut rule_applies = true;
            for j in 0..r.right.len() {
                let j = j as i32;

                let curr: u8 = if i + j + offset >= 0 && i + j + offset < self.stack.len() as i32 {
                    self.stack.get((i + j + offset) as usize).unwrap().token
                } else {
                    rule_applies = false;
                    break;
                };

                if self.grammar.non_terminals.contains(&curr) {
                    let mut token: Option<u8> = None;
                    for t in self.grammar.inverse_rewrite_rules.get(&curr).unwrap() {
                        if *t == *r.right.get(j as usize).unwrap() {
                            token = Some(*t);
                        }
                    }
                    if let Some(t) = token {
                        rewrites.insert(r.right[j as usize], t);
                    } else {
                        rule_applies = false;
                    }
                } else if curr != *r.right.get(j as usize).unwrap() {
                    rule_applies = false;
                    break;
                }
            }
            if rule_applies {
                if r.right.len() > longest as usize {
                    longest = r.right.len() as i32;

                    if rewrites.is_empty() {
                        rule = Some((*r).clone());
                    } else {
                        rule = Some((*r).clone());
                        apply_rewrites = rewrites.clone();
                        rewrites.clear();
                    }
                }
            }
        }
        if let Some(rule) = rule {
            if !apply_rewrites.is_empty() {
                for _ in 0..rule.right.len() {
                    let mut current = self.stack.get_mut((i + offset) as usize).unwrap();
                    let token = apply_rewrites.get(&current.token);
                    if let Some(token) = token {
                        current.token = *token;
                        if self.open_nodes.contains_key(&current.id) {
                            self.open_nodes.get_mut(&current.id).unwrap().symbol = *token;
                        }
                    }
                }
            }

            let mut parent = Node::new(rule.left, None);
            for _ in 0..rule.right.len() {
                let current = self.stack.remove((i + offset) as usize);
                if self.open_nodes.contains_key(&current.id) {
                    let sub_tree = self.open_nodes.remove(&current.id).unwrap();
                    parent.append_child(sub_tree);
                } else {
                    let leaf = Node::new(current.token, None);
                    parent.append_child(leaf);
                }
            }

            let left = TokenGrammarTuple::new(rule.left, Associativity::Undefined, self);
            self.open_nodes.insert(left.id, parent);
            self.stack.insert((i + offset) as usize, left);
            debug!("{} Reduce", self.iteration);
            self.should_reconsume = true;
        }
    }

    pub fn print_stack(&self) {
        let mut output = String::new();
        for i in &self.stack {
            let x = match i.associativity {
                Associativity::Left => '<',
                Associativity::Right => '>',
                Associativity::Equal => '=',
                Associativity::Undefined => '?',
                Associativity::None => '!',
            };
            output.push_str(
                format!(
                    "({:?}, {}) ",
                    self.grammar.token_raw.get(&i.token).unwrap(),
                    x
                )
                .as_str(),
            );
        }
        debug!("{} Stack: {}", self.iteration, output);
    }

    pub fn collect_parse_tree(self) -> Result<ParseTree, Box<dyn Error>> {
        if self.open_nodes.len() == 1 {
            let mut nodes: Vec<Node> = self.open_nodes.into_iter().map(|(_, v)| v).collect();
            return Ok(ParseTree::new(nodes.remove(0), self.grammar));
        } else {
            panic!("Cannot create parse tree.");
        }
    }
}
