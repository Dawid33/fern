use crate::grammar::error::GrammarError;
use crate::grammar::printing::print_op_table;
use crate::grammar::reader::TokenTypes::{NonTerminal, Terminal};
pub use crate::grammar::reader::{RawGrammar, TokenTypes};
use crate::grammar::Associativity::{Equal, Left, Right};
use log::{debug, info, trace};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::HashMap;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::error::Error;
use std::fmt::{format, Debug};
use std::fs;
use std::fs::File;
use std::hash::Hash;
use std::io::{Read, Seek, Write};
use crate::reader::ReductionTree;

mod error;
pub mod printing;
pub mod reader;
pub mod transform;

pub type Token = u16;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rule {
    pub left: Token,
    pub right: Vec<Token>,
    pub nesting_rules: Vec<Vec<i16>>,
}

impl Rule {
    pub fn new() -> Self {
        Self {
            left: 0,
            right: Vec::new(),
            nesting_rules: Vec::new()
        }
    }
    pub fn from(left: Token) -> Self {
        Self {
            left,
            right: Vec::new(),
            nesting_rules: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[allow(unused)]
pub enum Associativity {
    None,
    Left,
    Right,
    Equal,
    Undefined,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpGrammar {
    pub non_terminals: Vec<Token>,
    pub terminals: Vec<Token>,
    pub delim: Token,
    pub axiom: Token,
    pub inverse_rewrite_rules: HashMap<Token, Vec<Token>>,
    pub rules: Vec<Rule>,
    pub token_types: HashMap<Token, TokenTypes>,
    pub token_raw: HashMap<Token, String>,
    pub token_reverse: HashMap<String, (Token, TokenTypes)>,
    pub ast_rules: Vec<Rule>,
    pub new_non_terminals_subset: Vec<Token>,
    pub new_non_terminal_reverse : HashMap<Token, BTreeSet<Token>>,
    pub reduction_tree: ReductionTree,
    pub new_reduction_tree: ReductionTree,
    pub foobar: HashMap<Token, ReductionTree>,
    pub old_axiom: Token,
    op_table: HashMap<Token, HashMap<Token, Associativity>>,
}

#[allow(unused)]
impl OpGrammar {
    pub fn from(path: &str) -> OpGrammar {
        let raw = RawGrammar::from(path).unwrap();
        OpGrammar::new(raw).unwrap()
    }

    pub fn new(mut g: RawGrammar) -> Result<OpGrammar, GrammarError> {
        let mut inverse_rewrite_rules: HashMap<Token, Vec<Token>> = HashMap::new();
        let mut op_table: HashMap<Token, HashMap<Token, Associativity>> = HashMap::new();

        let delim = g.gen_id();

        g.token_raw.insert(delim, String::from("_DELIM"));
        g.token_reverse.insert(String::from("_DELIM"), (delim, NonTerminal));

        // Validate that the grammar is in OPG form
        let repeated_rules = g.get_repeated_rhs();
        if let Some(repeated_rules) = repeated_rules {
            return Err(GrammarError::from("Cannot build OP Grammar from grammar with repeated right hand side.".to_string()));
        }

        let mut rewrite_rules: HashMap<Token, Vec<Token>> = HashMap::new();
        for t in &g.non_terminals {
            rewrite_rules.insert(*t, Vec::new());
        }
        let mut modified = true;
        while modified {
            modified = false;
            for r in &g.rules {
                let token = r.right.get(0).unwrap();
                if r.right.len() != 1 || g.terminals.contains(token) {
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

        for t in &g.non_terminals {
            inverse_rewrite_rules.insert(*t, vec![*t]);
        }
        for t in &g.non_terminals {
            for t1 in rewrite_rules.get(t).unwrap() {
                inverse_rewrite_rules.get_mut(t1).unwrap().push(*t);
            }
        }

        debug!("INVERSE REWRITE RULES");
        for row in inverse_rewrite_rules.keys() {
            let mut row_full_raw = String::new();
            row_full_raw.push_str(Self::list_to_string(inverse_rewrite_rules.get(row).unwrap(), &g.token_raw).as_str());
            debug!(
                "{:?} -> {:?}",
                g.token_raw.get(row).unwrap(),
                row_full_raw,
            );
        }

        let mut first_ops: HashMap<Token, HashSet<Token>> = HashMap::new();
        let mut last_ops: HashMap<Token, HashSet<Token>> = HashMap::new();

        for r in &g.rules {
            if g.non_terminals.contains(&r.left) {
                if r.right.len() > 0 {
                    for s in &r.right {
                        if g.terminals.contains(&s) {
                            if !first_ops.contains_key(&r.left) {
                                first_ops.insert(r.left, HashSet::from([*s]));
                            } else {
                                first_ops.get_mut(&r.left).unwrap().insert(*s);
                            }
                            break;
                        }
                    }

                    for i in (0..r.right.len()).rev() {
                        if g.terminals.contains(&r.right[i]) {
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
            for r in &g.rules {
                if g.non_terminals.contains(&r.left) {
                    if r.right.len() > 0 {
                        if g.non_terminals.contains(&r.right[0]) {
                            if first_ops.contains_key(&r.right[0]) {
                                let bs = first_ops.get_mut(&r.right[0]).unwrap().clone();
                                if !first_ops.contains_key(&r.left) {
                                    did_something = true;
                                    first_ops.insert(r.left, HashSet::from_iter(bs.clone().into_iter()));
                                } else if !first_ops.get(&r.left).unwrap().is_superset(&bs) {
                                    did_something = true;
                                    for x in bs {
                                        first_ops.get_mut(&r.left).unwrap().insert(x);
                                    }
                                }
                            }
                        }

                        if g.non_terminals.contains(&r.right[r.right.len() - 1]) {
                            if last_ops.contains_key(&r.right[r.right.len() - 1]) {
                                let bs = last_ops.get(&r.right[r.right.len() - 1]).unwrap().clone();
                                if !last_ops.contains_key(&r.left) {
                                    did_something = true;
                                    last_ops.insert(r.left, HashSet::from_iter(bs.clone().into_iter()));
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
        let mut largest = 0;
        first_ops.keys().for_each(|x| {
            let s_len = g.token_raw.get(x).unwrap().len();
            if s_len > largest {
                largest = s_len
            }
        });
        for row in first_ops.keys() {
            let row_full_raw: Vec<&String> = first_ops
                .get(row)
                .unwrap()
                .iter()
                .map(|row_item| g.token_raw.get(row_item).unwrap())
                .collect();
            debug!(
                "{:s_len$} : {:?}",
                g.token_raw.get(row).unwrap(),
                row_full_raw,
                s_len = largest
            );
        }

        debug!("LAST OP");
        largest = 0;
        last_ops.keys().for_each(|x| {
            let s_len = g.token_raw.get(x).unwrap().len();
            if s_len > largest {
                largest = s_len
            }
        });
        for row in last_ops.keys() {
            let row_full_raw: Vec<&String> = last_ops
                .get(row)
                .unwrap()
                .iter()
                .map(|row_item| g.token_raw.get(row_item).unwrap())
                .collect();
            debug!(
                "{:s_len$} : {:?}",
                g.token_raw.get(row).unwrap(),
                row_full_raw,
                s_len = largest
            );
        }

        let mut template: HashMap<Token, Associativity> = HashMap::new();
        for t in &g.terminals {
            template.insert(*t, Associativity::None);
        }

        for t in &g.terminals {
            op_table.insert(*t, template.clone());
        }

        for r in &g.rules {
            for i in 0..r.right.len() {
                if i + 1 < r.right.len() {
                    if g.terminals.contains(r.right.get(i).unwrap())
                        && g.terminals.contains(r.right.get(i + 1).unwrap())
                    {
                        op_table
                            .get_mut(r.right.get(i).unwrap())
                            .unwrap()
                            .insert(*r.right.get(i + 1).unwrap(), Equal);
                    }
                    if g.terminals.contains(r.right.get(i).unwrap())
                        && g.non_terminals.contains(r.right.get(i + 1).unwrap())
                    {
                        if first_ops.contains_key(r.right.get(i + 1).unwrap()) {
                            let first_op_a = first_ops.get(r.right.get(i + 1).unwrap()).unwrap();
                            for q2 in first_op_a {
                                op_table.get_mut(r.right.get(i).unwrap()).unwrap().insert(*q2, Left);
                            }
                        }
                    }
                    if g.non_terminals.contains(r.right.get(i).unwrap())
                        && g.terminals.contains(r.right.get(i + 1).unwrap())
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
                        if g.terminals.contains(r.right.get(i).unwrap())
                            && g.non_terminals.contains(r.right.get(i + 1).unwrap())
                            && g.terminals.contains(r.right.get(i + 2).unwrap())
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

        op_table.insert(delim, template.clone().into_iter().map(|(t, a)| -> (Token, Associativity) {return (t, Associativity::Right);}).collect());
        for x in op_table.values_mut() {
            x.insert(delim, Associativity::Right);
        }
        op_table.get_mut(&delim).unwrap().insert(delim, Associativity::Equal);
        g.terminals.push(delim);

        print_op_table(&g.token_raw, &g.token_reverse, &g.terminals, &op_table);
        
        let mut tree = ReductionTree::new();
        for r in &g.rules {
            tree.add_rule(r);
        }

        Ok(OpGrammar {
            token_raw: g.token_raw,
            token_types: g.token_types,
            rules: g.rules,
            terminals: g.terminals,
            non_terminals: g.non_terminals,
            axiom: g.axiom,
            delim,
            inverse_rewrite_rules,
            op_table,
            ast_rules: g.ast_rules,
            token_reverse: g.token_reverse,
            new_non_terminal_reverse : g.new_non_terminal_reverse,
            new_non_terminals_subset: g.new_non_terminals_subset,
            reduction_tree: g.reduction_tree,
            new_reduction_tree: tree,
            foobar: g.foobar,
            old_axiom: g.old_axiom,
        })
    }

    fn token_list_to_string(value: &Vec<Token>, token_raw: &HashMap<Token, String>) -> Vec<String> {
        let mut output = Vec::new();
        for t in value {
            output.push(token_raw.get(t).unwrap().clone());
        }
        output
    }

    pub fn list_to_string(list: &Vec<Token>, token_raw: &HashMap<Token, String>) -> String {
        let mut sorted = Vec::new();
        for t in list {
            sorted.push(token_raw.get(t).unwrap().as_str());
        }
        // sorted.sort();
        let mut b = String::new();
        let mut iter = sorted.iter();
        if let Some(t) = iter.next() {
            b.push_str(format!("_{}", t).as_str());
        }
        while let Some(t) = iter.next() {
            b.push_str(format!("__{}", t).as_str());
        }
        b
    }

    pub fn to_file(&self, path: &str) {
        let mut f = File::create(path).unwrap();
        for t in &self.non_terminals {
            f.write(format!("%nonterminal {}\n", self.token_raw.get(&t).unwrap()).as_bytes());
        }

        f.write(format!("\n%axiom {}\n\n", self.token_raw.get(&self.axiom).unwrap()).as_bytes());

        for t in &self.terminals {
            f.write(format!("%terminal {}\n", self.token_raw.get(&t).unwrap()).as_bytes());
        }

        f.write("\n%%\n\n".as_bytes());

        let mut map: HashMap<Token, Vec<&Vec<Token>>> = HashMap::new();
        for r in &self.rules {
            if map.contains_key(&r.left) {
                map.get_mut(&r.left).unwrap().push(&r.right);
            } else {
                map.insert(r.left, vec![&r.right]);
            }
        }

        for (left, right) in map.iter() {
            let left = format!("{} : ", self.token_raw.get(left).unwrap());
            f.write(left.as_bytes());

            let mut rhs_list = right.iter();

            if let Some(rhs) = rhs_list.next() {
                let mut rhs_string = String::new();
                for x in rhs.iter() {
                    rhs_string.push_str(self.token_raw.get(x).unwrap());
                    rhs_string.push(' ');
                }
                rhs_string.push('\n');
                f.write(rhs_string.as_bytes());
            }

            while let Some(rhs) = rhs_list.next() {
                let mut rhs_string = String::new();
                rhs_string.push_str("\t| ");
                for x in rhs.iter() {
                    rhs_string.push_str(self.token_raw.get(x).unwrap());
                    rhs_string.push(' ');
                }
                rhs_string.push('\n');
                f.write(rhs_string.as_bytes());
            }
            f.write("\t;\n\n".as_bytes());
        }
    }

    pub fn get_precedence(&self, left: Token, right: Token) -> Associativity {
        return self.op_table.get(&left).unwrap().get(&right).unwrap().clone();
    }
}
