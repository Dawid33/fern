pub mod json;

use crate::grammar::{Associativity, OpGrammar, Rule, Token};
use log::{debug, error, info, warn};
use std::any::Any;
use std::collections::{HashMap, LinkedList, VecDeque};
use std::error::Error;
use std::hint::unreachable_unchecked;
use std::io::ErrorKind::AlreadyExists;
use std::io::Read;
use std::ops::Add;
use std::panic::{resume_unwind, set_hook};
use std::thread::current;
use std::time::Duration;
use tokio::time::Instant;

#[allow(unused)]
pub struct ParseTree {
    pub g: OpGrammar,
    root: Node,
}

impl ParseTree {
    #[allow(unused)]
    pub fn new(root: Node, g: OpGrammar) -> Self {
        Self { g, root }
    }
}

#[derive(Clone)]
pub struct Node {
    symbol: Token,
    data: Option<String>,
    children: Vec<Node>,
}

impl Node {
    pub fn new(symbol: Token, data: Option<String>) -> Self {
        Self {
            symbol,
            data,
            children: Vec::new(),
        }
    }
    pub fn prepend_child(&mut self, other: Node) {
        self.children.insert(0, other);
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
struct TokenGrammarTuple {
    token: Token,
    id: u64,
    associativity: Associativity,
}

impl TokenGrammarTuple {
    pub fn new(token: Token, associativity: Associativity, parser: &mut ParallelParser) -> Self {
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
        let mut child_count_stack: Vec<(i32, i32)> = vec![(0, (self.root.children.len() - 1) as i32)];
        let mut b = String::new();

        b.push_str(format!("{}", self.g.token_raw.get(&self.root.symbol).unwrap()).as_str());
        info!("{}", b);
        b.clear();
        while !node_stack.is_empty() {
            let current = node_stack.pop().unwrap();
            let (mut current_child, max_child) = child_count_stack.pop().unwrap();

            while current.children.len() > 0 && current_child <= max_child {
                for i in 0..child_count_stack.len() {
                    let (current, max) = child_count_stack.get(i).unwrap();
                    if *current <= *max {
                        b.push_str("| ");
                    } else {
                        b.push_str("  ");
                    }
                }
                if current_child != max_child {
                    b.push_str(format!(
                        "├─{}",
                        self.g
                            .token_raw
                            .get(&current.children.get(current_child as usize).unwrap().symbol)
                            .unwrap()
                    ).as_str());
                } else {
                    b.push_str(format!(
                        "└─{}",
                        self.g
                            .token_raw
                            .get(&current.children.get(current_child as usize).unwrap().symbol)
                            .unwrap()
                    ).as_str());
                }
                info!("{}", b);
                b.clear();
                if !current
                    .children
                    .get(current_child as usize)
                    .unwrap()
                    .children
                    .is_empty()
                {
                    node_stack.push(current);
                    let child = current.children.get(current_child as usize).unwrap();
                    current_child += 1;
                    node_stack.push(child);
                    child_count_stack.push((current_child, max_child));
                    child_count_stack.push((0, (child.children.len() - 1) as i32));
                    break;
                }
                current_child += 1;
            }
        }
    }
}

pub struct ParallelParser {
    stack: Vec<TokenGrammarTuple>,
    pub g: OpGrammar,
    open_nodes: HashMap<u64, Node>,
    should_reconsume: bool,
    highest_id: u64,
    iteration: u64,
    pub time_spent_rule_searching: Duration,
}

impl ParallelParser {
    pub fn new(grammar: OpGrammar, threads: usize) -> Self {
        let _ = threads;
        let parser = Self {
            stack: Vec::new(),
            g: grammar,
            should_reconsume: false,
            open_nodes: HashMap::new(),
            highest_id: 0,
            iteration: 0,
            time_spent_rule_searching: Duration::new(0, 0),
        };

        return parser;
    }

    pub fn parse(&mut self, tokens: LinkedList<Vec<Token>>) {
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

    fn consume_token(&mut self, token: &Token) -> Result<(), Box<dyn Error>> {
        if self.stack.is_empty() {
            let t = TokenGrammarTuple::new(*token, Associativity::Left, self);
            self.stack.push(t);
            return Ok(());
        }

        loop {
            self.iteration += 1;
            self.should_reconsume = false;

            let mut output = String::new();
            for (key, node) in &self.open_nodes {
                output.push_str(format!("({:?} {:?}) ", key, self.g.token_raw.get(&node.symbol).unwrap()).as_str());
            }
            debug!("{} Open nodes: {}", self.iteration, output);
            self.print_stack();

            let mut y: Option<TokenGrammarTuple> = None;
            for element in &self.stack {
                if self.g.terminals.contains(&element.token) {
                    y = Some(*element);
                }
            }

            let y = if self.g.delim != *token {
                if let None = y {
                    let t = TokenGrammarTuple::new(*token, Associativity::Left, self);
                    self.stack.push(t);
                    return Ok(());
                }
                y.unwrap()
            } else {
                TokenGrammarTuple::new(self.g.delim, Associativity::Left, self)
            };

            let precedence = if *token == self.g.delim {
                Associativity::Right
            } else {
                self.g.get_precedence(y.token, *token)
            };

            if precedence == Associativity::None {
                panic!(
                    "No precedence between y = {} and token = {}, which is probably a user grammar error",
                    self.g.token_raw.get(&y.token).unwrap(),
                    self.g.token_raw.get(&token).unwrap()
                )
            }

            debug!(
                "{} Applying {:?} {:?}",
                self.iteration,
                self.g.token_raw.get(token).unwrap(),
                precedence
            );

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

            if self.g.non_terminals.contains(token) {
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

                if i < 0 && *token != self.g.delim {
                    let t = TokenGrammarTuple::new(*token, Associativity::Right, self);
                    self.stack.push(t);
                    debug!("{}, Append", self.iteration);
                    return Ok(());
                } else if i - 1 >= 0 {
                    let xi_minus_one = self.stack.get((i - 1) as usize).unwrap();

                    if self.g.terminals.contains(&xi_minus_one.token) {
                        self.process_terminal(i);
                    } else if self.g.non_terminals.contains(&xi_minus_one.token) {
                        self.process_non_terminal(i);
                    } else {
                        panic!("Should be able to reduce but cannot. Probably a parser bug.");
                    }
                } else {
                    self.process_terminal(0);
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
        let mut apply_rewrites: HashMap<Token, Token> = HashMap::new();
        let mut longest: i32 = 0;

        let now = Instant::now();
        for r in &self.g.rules {
            let mut rewrites: HashMap<Token, Token> = HashMap::new();
            let mut rule_applies = true;
            for j in 0..r.right.len() {
                let j = j as i32;

                let curr: Token = if i + j + offset >= 0 && i + j + offset < self.stack.len() as i32 {
                    self.stack.get((i + j + offset) as usize).unwrap().token
                } else {
                    rule_applies = false;
                    break;
                };

                if self.g.non_terminals.contains(&curr) {
                    let mut token: Option<Token> = None;
                    for t in self.g.inverse_rewrite_rules.get(&curr).unwrap() {
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

                    debug!("Found rule {:?}", self.g.token_raw.get(&r.left).unwrap());

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
        let time = now.elapsed();
        debug!("Time spend searching: {:?}", &time);
        self.time_spent_rule_searching = self.time_spent_rule_searching.add(time);

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

            // Take stuff off stack that will become new parents children.
            let mut children = Vec::new();
            // flatten any non-terminal children that have only one node.
            for _ in 0..rule.right.len() {
                let current = self.stack.remove((i + offset) as usize);
                if self.open_nodes.contains_key(&current.id) {
                    let mut sub_tree = self.open_nodes.remove(&current.id).unwrap();
                    Self::flatten(&mut sub_tree, &self.g);
                    children.push(sub_tree);
                } else {
                    let leaf = Node::new(current.token, None);
                    children.push(leaf);
                }
            }

            let mut ast_rule: Option<Rule> = None;
            for r in &self.g.ast_rules {
                let mut found_rule = true;
                for (i, t) in r.right.iter().enumerate() {
                    if r.right.len() != rule.right.len() {
                        found_rule = false;
                    } else if self.g.terminals.contains(t) {
                        if let Some(current) = rule.right.get(i) {
                            if *current != *t {
                                found_rule = false;
                                continue;
                            }
                        }
                    } else if let Some(current) = rule.right.get(i) {
                        if !self.g.non_terminals.contains(current) {
                            found_rule = false;
                            continue;
                        }
                    }
                }
                if found_rule {
                    ast_rule = Some(r.clone());
                    break;
                }
            }

            let mut parent = Node::new(rule.left, None);

            if let Some(ast_rule) = ast_rule {
                let mut depth = 1;
                let mut nodes: Vec<(Vec<_>, Node)> = Vec::new();
                let mut current: Option<&mut Node> = Some(&mut parent);
                for (i, f) in children.into_iter().enumerate() {
                    // let _ = ast_rule.nesting_rules.get(i).unwrap();
                    nodes.push((ast_rule.nesting_rules.get(i).unwrap().clone(), f));
                }

                loop {
                    let mut current_depth_nodes = Vec::new();
                    let mut left = Vec::new();
                    for x in nodes.into_iter() {
                        if x.0.len() == depth {
                            current_depth_nodes.push(x);
                        } else {
                            left.push(x);
                        }
                    }
                    nodes = left;

                    if current_depth_nodes.len() == 0 {
                        break;
                    }

                    let mut offset = 0;
                    for mut node in current_depth_nodes {
                        let mut current = &mut current.as_mut().unwrap().children;
                        let slice = &node.0[0..node.0.len() - 1];
                        for i in slice {
                            current = &mut current.get_mut(*i as usize).unwrap().children;
                        }

                        if current.len() < *node.0.last().unwrap() as usize {
                            if self.g.non_terminals.contains(&node.1.symbol) {
                                if !node.1.children.is_empty() {
                                    node.1.children.reverse();
                                    offset -= 1;
                                    for x in node.1.children {
                                        offset += 1;
                                        current.push(x);
                                    }
                                    for (nesting, _) in nodes.iter_mut() {
                                        if nesting.len() >= depth {
                                            if let Some(next_depth) = nesting.get_mut(depth - 1) {
                                                let correct_index = *node.0.last().unwrap() as usize;
                                                if *next_depth >= correct_index as i16 {
                                                    *next_depth += offset;
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                current.push(node.1);
                            }
                        } else {
                            if self.g.non_terminals.contains(&node.1.symbol) {
                                if !node.1.children.is_empty() {
                                    node.1.children.reverse();
                                    offset -= 1;
                                    for x in node.1.children {
                                        offset += 1;
                                        current.insert(*node.0.last().unwrap() as usize, x);
                                    }
                                    for (nesting, _) in nodes.iter_mut() {
                                        if nesting.len() >= depth {
                                            if let Some(next_depth) = nesting.get_mut(depth - 1) {
                                                let correct_index = *node.0.last().unwrap() as usize;
                                                if *next_depth >= correct_index as i16 {
                                                    *next_depth += offset;
                                                }
                                            }
                                        }
                                    }
                                }
                            } else {
                                current.insert((*node.0.last().unwrap() + offset) as usize, node.1);
                            }
                        }
                    }

                    depth += 1;
                }
            } else {
                // If there exists no rule for this then just chuck everything into parent.
                children.reverse();
                let mut iter = children.into_iter();
                while let Some(x) = iter.next() {
                    parent.prepend_child(x);
                }
            }

            let left = TokenGrammarTuple::new(rule.left, Associativity::Undefined, self);
            self.open_nodes.insert(left.id, parent);
            self.stack.insert((i + offset) as usize, left);
            debug!("{} Reduce", self.iteration);
            self.should_reconsume = true;
        } else if self.stack.len() > 0 && self.g.axiom == self.stack.get(0).unwrap().token {
            debug!("{} Reached axiom and finished parsing.", self.iteration);
        } else {
            warn!("{} Should probably reduce but didn't. Could be a bug / error.", self.iteration);
        }
    }

    fn flatten(mut n: &mut Node, g: &OpGrammar) {
        if n.children.len() == 1 && g.non_terminals.contains(&n.symbol) {
            let other = n.children.remove(0);
            n.symbol = other.symbol;
            n.data = other.data;
            n.children = other.children;
        } else {
            for next in &mut n.children {
                Self::flatten(next, g);
            }
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
            output.push_str(format!("({:?}, {}) ", self.g.token_raw.get(&i.token).unwrap(), x).as_str());
        }
        debug!("{} Stack: {}", self.iteration, output);
    }

    pub fn collect_parse_tree(self) -> Result<ParseTree, Box<dyn Error>> {
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

        if self.open_nodes.len() == 1 {
            let mut nodes: Vec<Node> = self.open_nodes.into_iter().map(|(_, v)| v).collect();
            let root = nodes.remove(0);
            return Ok(ParseTree::new(root, self.g));
        } else {
            panic!("Cannot create parse tree.");
        }
    }
}
