use crate::grammar::opg::{Associativity, OpGrammar, Rule, Token};
use crate::lexer::Data;
use crate::parsetree::{Id, ParseTree};
use log::{debug, error, info, trace, warn};
use std::any::Any;
use std::collections::{BTreeMap, BTreeSet, HashMap, LinkedList, VecDeque};
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
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Node {
    pub symbol: Token,
    pub data: Option<Data>,
    pub children: Vec<Node>,
}

impl Node {
    pub fn new(symbol: Token, data: Option<Data>) -> Self {
        Self {
            symbol,
            data,
            children: Vec::new(),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct TokenGrammarTuple {
    data: Option<Data>,
    pub token: Token,
    id: u64,
    tree_id: Option<Id>,
    associativity: Associativity,
}

impl TokenGrammarTuple {
    pub fn new(token: Token, associativity: Associativity, id: u64, data: Option<Data>) -> Self {
        Self {
            token,
            associativity,
            id,
            data,
            tree_id: None,
        }
    }
}

pub struct PartialParseTree {
    parser: Parser,
}

impl PartialParseTree {
    pub fn merge(&mut self, tree: PartialParseTree) {}
    pub fn into_tree(self) -> ParseTree {
        self.parser.tree
    }
}

pub struct Parser {
    tree: ParseTree,
    stack: Vec<TokenGrammarTuple>,
    pub g: OpGrammar,
    open_nodes: BTreeMap<u64, Node>,
    should_reconsume: bool,
    highest_id: u64,
    iteration: u64,
    terminals_set: bittyset::BitSet<Token>,
    non_terminals_set: bittyset::BitSet<Token>,
    pub time_spent_rule_searching: Duration,
}

impl Parser {
    pub fn new(grammar: OpGrammar) -> Self {
        let mut terminals_set = bittyset::BitSet::<Token>::new();
        grammar.terminals.iter().for_each(|x| {
            terminals_set.insert(*x as usize);
        });

        let mut non_terminals_set = bittyset::BitSet::<Token>::new();
        grammar.non_terminals.iter().for_each(|x| {
            non_terminals_set.insert(*x as usize);
        });

        let map = grammar
            .token_raw
            .clone()
            .into_iter()
            .map(|(k, v)| (k, v.rsplit_once('_').unwrap_or((v.as_str(), v.as_str())).1.to_string()))
            .collect();
        let parser = Self {
            stack: Vec::new(),
            tree: ParseTree::new(map),
            g: grammar,
            should_reconsume: false,
            open_nodes: BTreeMap::new(),
            highest_id: 0,
            iteration: 0,
            terminals_set,
            non_terminals_set,
            time_spent_rule_searching: Duration::new(0, 0),
        };

        return parser;
    }

    pub fn parse(&mut self, tokens: Vec<Token>, data: Vec<Data>) {
        let mut iter = data.into_iter();
        for (i, t) in tokens.iter().enumerate() {
            if let Some(data) = iter.next() {
                if data.token_index == i {
                    self.consume_token(*t, Some(data)).expect("Parser raised an exception.");
                } else {
                    self.consume_token(*t, None).expect("Parser raised an exception.");
                }
            } else {
                self.consume_token(*t, None).expect("Parser raised an exception.");
            }
        }
    }

    pub fn gen_id(&mut self) -> u64 {
        self.highest_id += 1;
        return self.highest_id;
    }

    pub fn push(&mut self, mut tuple: TokenGrammarTuple) {
        let id = self.tree.push(tuple.token);
        tuple.tree_id = Some(id);
        self.stack.push(tuple);
    }

    fn consume_token(&mut self, token: Token, data: Option<Data>) -> Result<(), Box<dyn Error>> {
        if self.stack.is_empty() {
            let t = TokenGrammarTuple::new(token, Associativity::Left, self.gen_id(), data);
            self.push(t);
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
                if self.terminals_set.contains(element.token as usize) {
                    y = Some(element.clone());
                }
            }

            let y = if self.g.delim != token {
                if let None = y {
                    let t = TokenGrammarTuple::new(token, Associativity::Left, self.gen_id(), data);
                    self.push(t);
                    return Ok(());
                }
                y.unwrap()
            } else {
                TokenGrammarTuple::new(self.g.delim, Associativity::Left, self.gen_id(), None)
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

            trace!("{} Applying {:?} {:?}", self.iteration, self.g.token_raw.get(&token).unwrap(), precedence);

            if precedence == Associativity::Left {
                let t = TokenGrammarTuple::new(token, Associativity::Left, self.gen_id(), data);
                self.push(t);
                debug!("{} Append", self.iteration);
                return Ok(());
            }

            if precedence == Associativity::Equal {
                let t = TokenGrammarTuple::new(token, Associativity::Equal, self.gen_id(), data);
                self.push(t);
                debug!("{} Append", self.iteration);
                return Ok(());
            }

            if self.non_terminals_set.contains(token as usize) {
                let t = TokenGrammarTuple::new(token, Associativity::Undefined, self.gen_id(), data);
                self.push(t);
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
                    self.push(t);
                    debug!("{}, Append", self.iteration);
                    return Ok(());
                } else if i - 1 >= 0 {
                    let xi_minus_one = self.stack.get((i - 1) as usize).unwrap();

                    if self.terminals_set.contains(xi_minus_one.token as usize) {
                        self.process_terminal(i);
                    } else if self.non_terminals_set.contains(xi_minus_one.token as usize) {
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
        let apply_rewrites: HashMap<Token, Token> = HashMap::new();
        // let longest: i32 = 0;

        // let now = Instant::now();
        // TODO: Make this into a slice without collecting into vec, probably implement custom iter.
        let iter: Vec<&Token> = (&self.stack[(i + offset) as usize..]).iter().map(|x| -> &Token { &x.token }).collect();
        let rule: Option<&Rule> = self.g.new_reduction_tree.match_rule(&iter[..], &self.g.token_raw);

        // let time = now.elapsed();
        // debug!("Time spend searching: {:?}", &time);
        // self.time_spent_rule_searching = self.time_spent_rule_searching.add(time);

        if let Some(rule) = rule {
            if !apply_rewrites.is_empty() {
                for _ in 0..rule.right.len() {
                    let current = self.stack.get_mut((i + offset) as usize).unwrap();
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
            for t in &rule.right {
                warn!("{:?}", self.tree.token_map.get(t));
            }
            for _ in 0..rule.right.len() {
                let current = self.stack.remove((i + offset) as usize);
                if self.open_nodes.contains_key(&current.id) {
                    self.open_nodes.remove(&current.id).unwrap();
                }
                children.push(current.tree_id.unwrap());
            }

            let p_id = self.tree.reduce(rule.left, &children);
            let parent = Node::new(rule.left, None);

            let mut left = TokenGrammarTuple::new(rule.left, Associativity::Undefined, self.gen_id(), None);
            left.tree_id = Some(p_id);
            for n in &mut self.stack {
                if n.tree_id.unwrap() >= p_id {
                    *n.tree_id.as_mut().unwrap() += 1;
                }
            }

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

    fn expand(n: &mut Node, p: &Parser) {
        trace!("Expanding: {}", p.g.token_raw.get(&n.symbol).unwrap());
        let term_list = p.g.new_non_terminal_reverse.get(&n.symbol);
        let term_list = if let Some(list) = term_list {
            list.iter().map(|x| *x).collect()
        } else {
            Vec::from([n.symbol])
        };
        n.symbol = *term_list.last().unwrap();
        for (_i, next) in n.children.iter_mut().enumerate() {
            if p.non_terminals_set.contains(next.symbol as usize) {
                Self::expand(next, p);
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
        trace!("{} Stack: {}", self.iteration, output);
    }

    pub fn collect_parse_tree(self) -> Result<PartialParseTree, Box<dyn Error>> {
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

        return Ok(PartialParseTree { parser: self });
    }
}
