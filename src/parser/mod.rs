use std::any::Any;
use std::collections::{HashMap, LinkedList};
use std::error::Error;
use std::io::ErrorKind::AlreadyExists;
use std::panic::{resume_unwind, set_hook};
use std::thread::current;
use crate::grammar::{Associativity, Grammar, Rule};
use crate::lexer::json::JsonToken;

#[allow(unused)]
pub struct ParseTree {
    root: Node
}

impl ParseTree {
    #[allow(unused)]
    pub fn new(root: Node) -> Self {
        Self {
           root,
        }
    }
}

pub struct Node {
    symbol: JsonToken,
    children: Option<Vec<Node>>,
}

impl Node {
    pub fn new (symbol: JsonToken) -> Self {
        Self { symbol, children: None }
    }
    pub fn append_child(&mut self, other: Node) {
        if let None = self.children {
            self.children = Some(Vec::new());
        }
        self.children.as_mut().unwrap().push(other);
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
struct TokenGrammarTuple {
    token: JsonToken,
    id: u64,
    associativity: Associativity
}

impl TokenGrammarTuple {
    pub fn new(token: JsonToken, associativity: Associativity, parser: &mut ParallelParser) -> Self {
        Self {
            token,
            associativity,
            id: parser.gen_id(),
        }
    }
}

pub struct ParallelParser {
    stack: Vec<TokenGrammarTuple>,
    pub grammar: Grammar<JsonToken>,
    open_nodes: HashMap<u64, Node>,
    should_reconsume: bool,
    highest_id: u64,
}

impl ParallelParser {
    pub fn new(grammar: Grammar<JsonToken>, threads: usize) -> Self {
        let _ = threads;
        let parser = Self {
            stack: Vec::new(),
            grammar,
            should_reconsume: false,
            open_nodes: HashMap::new(),
            highest_id: 0,
        };

        return parser;
    }

    pub fn parse(&mut self, tokens: &[JsonToken]) {
        for t in tokens {
            self.consume_token(&t).expect("Parser raised an exception.");
        }
    }

    pub fn gen_id(&mut self) -> u64 {
        self.highest_id += 1;
        return self.highest_id;
    }

    fn consume_token(&mut self, token: &JsonToken) -> Result<(), Box<dyn Error>>{

        if self.stack.is_empty() {
            let t = TokenGrammarTuple::new(*token, Associativity::Left, self);
            self.stack.push(t);
            return Ok(());
        }

        loop {
            self.should_reconsume = false;

            if self.stack.len() == 1 {
                if self.stack.get(0).unwrap().associativity == Associativity::Undefined {
                    println!("Done. Result: ");

                    println!("Open nodes");
                    for (id, _) in &self.open_nodes {
                        for t in &self.stack {
                            if t.id == *id  {
                                println!("\t{:?}", t.token);
                                break;
                            }
                        }
                    }
                    self.print_stack();
                    return Ok(());
                }
            }

            let mut y: Option<TokenGrammarTuple> = None;
            for element in &self.stack {
                if self.grammar.terminals.contains(&Self::standardize(element.token)) {
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
                self.grammar.get_precedence(Self::standardize(y.token), Self::standardize(*token))
            };

            if precedence == Associativity::None {
                return Err(Box::try_from("No precedence == user grammar error").unwrap());
            }

            // println!("Open nodes");
            // for (key, _) in &self.open_nodes {
            //     println!("\t{:?}", key.token);
            // }
            // println!("Applying {:?} {:?}", token, precedence);
            // self.print_stack();

            if precedence == Associativity::Left {
                let t = TokenGrammarTuple::new(*token, Associativity::Left, self);
                self.stack.push(t);
                // println!("Append\n");
                return Ok(());
            }

            if precedence == Associativity::Equal {
                let t = TokenGrammarTuple::new(*token, Associativity::Equal, self);
                self.stack.push(t);
                // println!("Append\n");
                return Ok(());
            }

            if self.grammar.non_terminals.contains(&Self::standardize(*token)) {
                let t = TokenGrammarTuple::new(*token, Associativity::Undefined, self);
                self.stack.push(t);
                // println!("Append\n");
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
                    // println!("Append\n");
                    return Ok(());
                } else {
                    if i - 1 >= 0 {
                        let xi_minus_one = self.stack.get((i - 1) as usize).unwrap();

                        if self.grammar.terminals.contains(&Self::standardize(xi_minus_one.token)) {
                            self.process_terminal(i);
                        } else if self.grammar.non_terminals.contains(&Self::standardize(xi_minus_one.token)) {
                            self.process_non_terminal(i);
                        } else {
                            return Err(Box::try_from("Should be able to reduce but cannot.").unwrap());
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

    fn standardize(t: JsonToken) -> JsonToken {
        match t {
            JsonToken::Number(_) => JsonToken::Number(0),
            JsonToken::Character(_) => JsonToken::Character(' '),
            _ => t
        }
    }
    fn process_terminal(&mut self, i: i32) { self.reduce_stack(i, 0); }

    fn process_non_terminal(&mut self, i: i32) { self.reduce_stack(i, -1); }

    fn reduce_stack(&mut self, i: i32, offset: i32) {
        let mut rule: Option<Rule<JsonToken>> = None;
        let mut apply_rewrites: HashMap<JsonToken, JsonToken> = HashMap::new();
        let mut longest: i32 = 0;

        for r in &self.grammar.rules {
            let mut rewrites: HashMap<JsonToken, JsonToken> = HashMap::new();
            let mut rule_applies = true;
            for j in 0..r.right.len() {
                let j = j as i32;

                let curr: JsonToken = if i + j + offset >= 0 && i + j + offset < self.stack.len() as i32 {
                    self.stack.get((i + j + offset) as usize).unwrap().token
                } else {
                    rule_applies = false;
                    break;
                };

                if self.grammar.non_terminals.contains(&Self::standardize(curr)) {
                    let mut token: Option<JsonToken> = None;
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
                } else if Self::standardize(curr) != *r.right.get(j as usize).unwrap() {
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

            let mut parent = Node::new(rule.left);
            for _ in 0..rule.right.len() {
                let current = self.stack.remove((i + offset) as usize);
                if self.open_nodes.contains_key(&current.id) {
                    let sub_tree = self.open_nodes.remove(&current.id).unwrap();
                    parent.append_child(sub_tree);
                } else {
                    let leaf = Node::new(current.token);
                    parent.append_child(leaf);
                }
            }

            let left = TokenGrammarTuple::new(rule.left, Associativity::Undefined, self);
            self.open_nodes.insert(left.id, parent);
            self.stack.insert((i + offset) as usize, left);
            // println!("Reduce\n");
            self.should_reconsume = true;
        }
    }

    pub fn print_stack(&self) {
        for i in &self.stack {
            let x = match i.associativity {
                Associativity::Left => '<',
                Associativity::Right => '>',
                Associativity::Equal => '=',
                Associativity::Undefined => '?',
                Associativity::None => '!'
            };
            print!("({:?}, {}) ", i.token, x);
        }
        println!();
    }

    pub fn collect_parse_tree(self) -> Result<ParseTree, Box<dyn Error>> {
        if self.open_nodes.len() == 1 {
            let mut nodes: Vec<Node>  = self.open_nodes.into_iter().map(|(_, v)| v).collect();
            return Ok(ParseTree::new(nodes.remove(0)));
        } else {
            return Err(Box::try_from("Cannot create parse tree.").unwrap());
        }
    }
}
