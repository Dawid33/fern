use std::collections::hash_map::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::fmt::Debug;
use std::fs;
use std::fs::File;
use std::hash::Hash;
use std::io::Read;
use log::debug;
use crate::grammar::Associativity::{Equal, Left, Right};
use crate::grammar::error::GrammarError;
use crate::grammar::reader::{read_grammar_file, TokenTypes};
use crate::grammar::reader::TokenTypes::{NonTerminal, Terminal};

pub mod reader;
mod error;

#[derive(Clone, Debug)]
pub struct Rule {
    pub left: u8,
    pub right: Vec<u8>,
}

impl Rule {
    pub fn new (left: u8) -> Self {
        Self {
            left,
            right: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
#[allow(unused)]
pub enum Associativity {
    None,
    Left,
    Right,
    Equal,
    Undefined,
}

#[derive(Clone, Debug)]
pub struct Grammar {
    pub non_terminals: Vec<u8>,
    pub terminals: Vec<u8>,
    pub delim: u8,
    pub axiom: u8,
    pub inverse_rewrite_rules: HashMap<u8, Vec<u8>>,
    pub rules: Vec<Rule>,
    pub token_types: HashMap<u8, TokenTypes>,
    pub token_raw: HashMap<u8, String>,
    pub tokens_reverse: HashMap<String, (u8, TokenTypes)>,
    op_table: HashMap<u8, HashMap<u8, Associativity>>,
}

#[allow(unused)]
impl Grammar {
    pub fn from(path: &str) -> Grammar {
        let mut file = fs::File::open(path).unwrap();
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();

        read_grammar_file(buf.as_str()).unwrap()
    }

    pub fn new(
        rules: Vec<Rule>,
        token_types: HashMap<u8, TokenTypes>,
        token_raw: HashMap<u8, String>,
        tokens_reverse: HashMap<String, (u8, TokenTypes)>,
        axiom: u8,
        delim: u8,
    ) -> Result<Grammar, GrammarError> {
        let mut inverse_rewrite_rules: HashMap<u8, Vec<u8>> = HashMap::new();
        let mut op_table: HashMap<u8, HashMap<u8, Associativity>> = HashMap::new();

        let mut non_terminals = Vec::new();
        let mut terminals = Vec::new();
        for (id, v) in &token_types {
            if *v == NonTerminal {
                non_terminals.push(*id);
            } else if *v == Terminal {
                terminals.push(*id);
            }
        }

        // Create re-write rules
        // TODO : Figure out how this actually works.
        let mut rewrite_rules: HashMap<u8, Vec<u8>> = HashMap::new();
        for t in &non_terminals {
            rewrite_rules.insert(*t, Vec::new());
        }
        let mut modified = true;
        while modified {
            modified = false;
            for r in &rules {
                let token = r.right.get(0).unwrap();
                if r.right.len() != 1 || terminals.contains(token) {
                    continue;
                }
                if !rewrite_rules.get_mut(&r.left).unwrap().contains(token) {
                    modified = true;
                    rewrite_rules.get_mut(&r.left).unwrap().push(*token);
                } else {
                    for ttoken in rewrite_rules.get(token).unwrap().clone() {
                        if !rewrite_rules.get(&r.left).unwrap().contains(&ttoken) {
                            modified = true;
                            rewrite_rules.get_mut(&r.left).unwrap().push(ttoken);
                        }
                    }
                }
            }
        }

        // Create inverse rewrite rules
        // TODO: Figure out what this is.
        for t in &non_terminals {
            inverse_rewrite_rules.insert(*t, vec![*t]);
        }
        for t in &non_terminals {
            for t1 in rewrite_rules.get(t).unwrap() {
                inverse_rewrite_rules.get_mut(t1).unwrap().push(*t);
            }
        }

        let mut first_ops: HashMap<u8, HashSet<u8>> = HashMap::new();
        let mut last_ops: HashMap<u8, HashSet<u8>> = HashMap::new();

        for r in &rules {
            if non_terminals.contains(&r.left) {
                if r.right.len() > 0 {
                    for s in &r.right {
                        if terminals.contains(&s) {
                            if !first_ops.contains_key(&r.left) {
                                first_ops.insert(r.left, HashSet::from([*s]));
                            } else {
                                first_ops.get_mut(&r.left).unwrap().insert(*s);
                            }
                            break;
                        }
                    }

                    // Possible error, check later
                    for i in (0..r.right.len()).rev() {
                        if terminals.contains(&r.right[i]) {
                            if !last_ops.contains_key(&r.left) {
                                last_ops.insert(r.left, HashSet::from([r.right[i]]));
                            } else {
                                last_ops.get_mut(&r.left).unwrap().insert(r.right[i]);
                            }
                            break;
                        }
                    }
                }
            }
        }

        let mut did_something: bool;
        loop {
            did_something = false;
            for r in &rules {
                if non_terminals.contains(&r.left) {
                    if r.right.len() > 0 {
                        if non_terminals.contains(&r.right[0]) {
                            if first_ops.contains_key(&r.right[0]) {
                                let bs = first_ops.get_mut(&r.right[0]).unwrap().clone();
                                if !first_ops.contains_key(&r.left) {
                                    did_something = true;
                                    first_ops
                                        .insert(r.left, HashSet::from_iter(bs.clone().into_iter()));
                                } else if !first_ops.get(&r.left).unwrap().is_superset(&bs) {
                                    did_something = true;
                                    for x in bs {
                                        first_ops.get_mut(&r.left).unwrap().insert(x);
                                    }
                                }
                            }
                        }

                        if non_terminals.contains(&r.right[r.right.len() - 1]) {
                            if last_ops.contains_key(&r.right[r.right.len() - 1]) {
                                let bs = last_ops.get(&r.right[r.right.len() - 1]).unwrap().clone();
                                if !last_ops.contains_key(&r.left) {
                                    did_something = true;
                                    last_ops
                                        .insert(r.left, HashSet::from_iter(bs.clone().into_iter()));
                                } else if !last_ops.get(&r.left).unwrap().is_superset(&bs) {
                                    did_something = true;
                                    for x in bs {
                                        last_ops.get_mut(&r.left).unwrap().insert(x);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if did_something {
                continue;
            } else {
                break;
            }
        }

        debug!("FIRST OP");
        for row in first_ops.keys() {
            debug!("{:?} : {:?}", row, first_ops.get(row));
        }
        debug!("");

        debug!("LAST OP");
        for row in last_ops.keys() {
            debug!("{:?} : {:?}", row, last_ops.get(row));
        }
        debug!("");

        let mut template: HashMap<u8, Associativity> = HashMap::new();
        for t in &terminals {
            template.insert(*t, Associativity::None);
        }

        for t in &terminals {
            op_table.insert(*t, template.clone());
        }

        for r in &rules {
            for i in 0..r.right.len() {
                if i + 1 < r.right.len() {
                    if terminals.contains(r.right.get(i).unwrap())
                        && terminals.contains(r.right.get(i + 1).unwrap())
                    {
                        op_table
                            .get_mut(r.right.get(i).unwrap())
                            .unwrap()
                            .insert(*r.right.get(i + 1).unwrap(), Equal);
                    }
                    if terminals.contains(r.right.get(i).unwrap())
                        && non_terminals.contains(r.right.get(i + 1).unwrap())
                    {
                        if first_ops.contains_key(r.right.get(i + 1).unwrap()) {
                            let first_op_a = first_ops.get(r.right.get(i + 1).unwrap()).unwrap();
                            for q2 in first_op_a {
                                op_table
                                    .get_mut(r.right.get(i).unwrap())
                                    .unwrap()
                                    .insert(*q2, Left);
                            }
                        }
                    }
                    if non_terminals.contains(r.right.get(i).unwrap())
                        && terminals.contains(r.right.get(i + 1).unwrap())
                    {
                        if last_ops.contains_key(r.right.get(i).unwrap()) {
                            let last_op_a = last_ops.get(r.right.get(i).unwrap()).unwrap();
                            for q2 in last_op_a {
                                op_table
                                    .get_mut(q2)
                                    .unwrap()
                                    .insert(*r.right.get(i + 1).unwrap(), Right);
                            }
                        }
                    }
                    if i + 2 < r.right.len() {
                        if terminals.contains(r.right.get(i).unwrap())
                            && non_terminals.contains(r.right.get(i + 1).unwrap())
                            && terminals.contains(r.right.get(i + 2).unwrap())
                        {
                            op_table
                                .get_mut(r.right.get(i).unwrap())
                                .unwrap()
                                .insert(*r.right.get(i + 2).unwrap(), Equal);
                        }
                    }
                }
            }
        }

        debug!("{:<16}", "");
        for row in &terminals {
            debug!("{:16}", format!("{:?}", row));
        }
        debug!("");

        for row in &terminals {
            debug!("{:16}", format!("{:?}", row));
            let curr_row = op_table.get(row).unwrap();
            for col in &terminals {
                debug!("{:16}", format!("{:?}", curr_row.get(col).unwrap()));
            }
            debug!("");
        }
        debug!("");

        Ok(Grammar {
            token_raw,
            token_types,
            rules,
            terminals,
            non_terminals,
            axiom,
            delim,
            inverse_rewrite_rules,
            op_table,
            tokens_reverse,
        })
    }

    pub fn get_precedence(&self, left: u8, right: u8) -> Associativity {
        return self
            .op_table
            .get(&left)
            .unwrap()
            .get(&right)
            .unwrap()
            .clone();
    }
}