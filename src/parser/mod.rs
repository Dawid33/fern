pub mod json;
pub mod fern;

use crate::grammar::{Associativity, OpGrammar, Rule, Token};
use log::{debug, error, info, warn};
use std::any::Any;
use std::collections::{BTreeSet, HashMap, LinkedList, VecDeque};
use std::error::Error;
use std::hint::unreachable_unchecked;
use std::io::ErrorKind::AlreadyExists;
use std::io::Read;
use std::marker::PhantomData;
use std::ops::Add;
use std::panic::{resume_unwind, set_hook};
use std::slice::Iter;
use std::sync::mpsc::channel;
use std::thread::current;
use std::time::Duration;
use tokio::time::Instant;
use crate::lexer::fern::FernData;
use crate::lua::LuaData::NoData;

#[allow(unused)]
pub trait ParseTree<T> {
    fn new(root: Node<T>, g: OpGrammar) -> Self;
    fn print(&self);
}


#[derive(Clone)]
pub struct Node<T> {
    pub symbol: Token,
    data: Option<T>,
    pub children: Vec<Node<T>>,
}

impl<T> Node<T> {
    pub fn new(symbol: Token, data: Option<T>) -> Self {
        Self {
            symbol,
            data,
            children: Vec::new(),
        }
    }
    pub fn prepend_child(&mut self, other: Node<T>) {
        self.children.insert(0, other);
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct TokenGrammarTuple<T>
where T: Clone {
    data: T,
    pub token: Token,
    id: u64,
    associativity: Associativity,
}

impl<T> TokenGrammarTuple<T>
where T: Clone {
    pub fn new(token: Token, associativity: Associativity, id: u64, data: T) -> Self {
        Self {
            token,
            associativity,
            id,
            data,
        }
    }
}

pub struct ParallelParser<T>
where T: Clone {
    stack: Vec<TokenGrammarTuple<T>>,
    pub g: OpGrammar,
    open_nodes: HashMap<u64, Node<T>>,
    should_reconsume: bool,
    highest_id: u64,
    iteration: u64,
    pub time_spent_rule_searching: Duration,
}

impl<T> ParallelParser<T>
    where T : Default + Clone
{
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

    pub fn parse(&mut self, tokens: LinkedList<Vec<(Token, T)>>) {
        for t in tokens {
            for t in t {
                self.consume_token(t.0, t.1).expect("Parser raised an exception.");
            }
        }
    }

    pub fn gen_id(&mut self) -> u64 {
        self.highest_id += 1;
        return self.highest_id;
    }

    fn consume_token(&mut self, token: Token, data: T) -> Result<(), Box<dyn Error>> {
        if self.stack.is_empty() {
            let t = TokenGrammarTuple::new(token, Associativity::Left, self.gen_id(), data);
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

            let mut y: Option<TokenGrammarTuple<T>> = None;
            for element in &self.stack {
                if self.g.terminals.contains(&element.token) {
                    y = Some(element.clone());
                }
            }

            let y = if self.g.delim != token {
                if let None = y {
                    let t = TokenGrammarTuple::new(token, Associativity::Left, self.gen_id(), data);
                    self.stack.push(t);
                    return Ok(());
                }
                y.unwrap()
            } else {
                TokenGrammarTuple::new(self.g.delim, Associativity::Left, self.gen_id(), T::default())
            };

            if let Some(t) = self.stack.get(0) {
                if t.token == self.g.axiom && y.token == self.g.delim {
                    return Ok(());
                }
            }

            let precedence = if token == self.g.delim {
                Associativity::Right
            } else {
                self.g.get_precedence(y.token, token)
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
                self.g.token_raw.get(&token).unwrap(),
                precedence
            );

            if precedence == Associativity::Left {
                let t = TokenGrammarTuple::new(token, Associativity::Left, self.gen_id(), data);
                self.stack.push(t);
                debug!("{} Append", self.iteration);
                return Ok(());
            }

            if precedence == Associativity::Equal {
                let t = TokenGrammarTuple::new(token, Associativity::Equal, self.gen_id(), data);
                self.stack.push(t);
                debug!("{} Append", self.iteration);
                return Ok(());
            }

            if self.g.non_terminals.contains(&token) {
                let t = TokenGrammarTuple::new(token, Associativity::Undefined, self.gen_id(), data);
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

                if i < 0 && token != self.g.delim {
                    let t = TokenGrammarTuple::new(token, Associativity::Right, self.gen_id(), data);
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
        let mut apply_rewrites: HashMap<Token, Token> = HashMap::new();
        let mut longest: i32 = 0;

        let now = Instant::now();
        // TODO: Make this into a slice without collecting into vec, probably implement custom iter.
        let iter: Vec<&Token> = (&self.stack[(i + offset) as usize..]).iter().map(|x| -> &Token {
            &x.token
        }).collect();
        let mut rule: Option<&Rule> = self.g.new_reduction_tree.match_rule(&iter[..], &self.g.token_raw);

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
                    children.push(sub_tree);
                } else {
                    let leaf = Node::new(current.token, Some(current.data));
                    children.push(leaf);
                }
            }

            let mut parent = Node::new(rule.left, None);
            parent.children.append(&mut children);

            let left = TokenGrammarTuple::new(rule.left, Associativity::Undefined, self.gen_id(), T::default());
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

    fn expand(mut n: &mut Node<T>, g: &OpGrammar, expected: Option<Token>) {
        info!("Expanding: {}", g.token_raw.get(&n.symbol).unwrap());
        let term_list = g.new_non_terminal_reverse.get(&n.symbol);
        let term_list = if let Some(list) = term_list {
            list.iter().map(|x|{*x}).collect()
        } else {
            Vec::from([n.symbol])
        };
        for possible_non_term in term_list.iter().rev() {
            info!("Trying : {}", g.token_raw.get(possible_non_term).unwrap());
            let tree = g.foobar.get(possible_non_term).unwrap();
            if let Some(r) = tree.disambiguate(&n, g, expected) {
                info!("Selected : {}", g.token_raw.get(possible_non_term).unwrap());
                n.symbol = r.left;
                for (i, next) in n.children.iter_mut().enumerate() {
                    if g.non_terminals.contains(&next.symbol) {
                        Self::expand(next, g, Some(*r.right.get(i).unwrap()));
                    }
                }
                break;
            } else {
                info!("Failed to disambiguate token.");
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

    pub fn collect_parse_tree<U: ParseTree<T>>(self) -> Result<U, Box<dyn Error>> {
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
            let mut nodes: Vec<Node<T>> = self.open_nodes.into_iter().map(|(_, v)| v).collect();
            let mut root = nodes.remove(0);
            for child in &mut root.children {
                Self::expand(child, &self.g, None);
            }
            return Ok(U::new(root, self.g));
        } else {
            panic!("Cannot create parse tree.");
        }
    }
}
