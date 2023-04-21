use crate::grammar::error::GrammarError;
use crate::grammar::reader::TokenTypes::{NonTerminal, Terminal};
use crate::grammar::reader::{read_grammar_file, TokenTypes};
use crate::grammar::Associativity::{Equal, Left, Right};
use log::{debug, error, info, log_enabled, trace, warn};
use std::collections::hash_map::HashMap;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::error::Error;
use std::fmt::{format, Debug};
use std::fs;
use std::fs::File;
use std::hash::Hash;
use std::io::{Read, Write};

mod error;
pub mod reader;

pub type Token = u16;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Rule {
    pub left: Token,
    pub right: Vec<Token>,
}

impl Rule {
    pub fn new() -> Self {
        Self {
            left: 0,
            right: Vec::new(),
        }
    }
    pub fn from(left: Token) -> Self {
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
    pub non_terminals: Vec<Token>,
    pub terminals: Vec<Token>,
    pub delim: Token,
    pub axiom: Token,
    pub inverse_rewrite_rules: HashMap<Token, Vec<Token>>,
    pub rules: Vec<Rule>,
    pub token_types: HashMap<Token, TokenTypes>,
    pub token_raw: HashMap<Token, String>,
    pub tokens_reverse: HashMap<String, (Token, TokenTypes)>,
    op_table: HashMap<Token, HashMap<Token, Associativity>>,
}

#[allow(unused)]
impl Grammar {
    pub fn from(path: &str) -> Grammar {
        let mut file = fs::File::open(path).unwrap();
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();

        read_grammar_file(buf.as_str()).unwrap()
    }

    fn get_repeated_rhs(rules: &Vec<Rule>) -> Option<HashMap<Vec<Token>, Vec<Rule>>> {
        let mut repeated_rules: HashMap<Vec<Token>, Vec<Rule>> = HashMap::new();
        let mut rhs_rule_map: HashMap<Vec<Token>, Vec<Rule>> = HashMap::new();
        for r in rules {
            if !rhs_rule_map.contains_key(&r.right) {
                rhs_rule_map.insert(r.right.clone(), Vec::from([r.clone()]));
            } else {
                rhs_rule_map.get_mut(&r.right).unwrap().push(r.clone());
            }
        }
        for (rhs, collected_rules) in rhs_rule_map {
            if collected_rules.len() > 1 {
                repeated_rules.insert(rhs, collected_rules);
            }
        }
        if repeated_rules.is_empty() {
            None
        } else {
            Some(repeated_rules)
        }
    }

    fn token_list_to_string(value: &Vec<Token>, token_raw: &HashMap<Token, String>) -> Vec<String> {
        let mut output = Vec::new();
        for t in value {
            output.push(token_raw.get(t).unwrap().clone());
        }
        output
    }

    fn add_new_rules(
        dict_rules_for_iteration: &mut HashMap<Vec<BTreeSet<Token>>, BTreeSet<Token>>,
        key_rhs: &[Token],
        value_lhs: &BTreeSet<Token>,
        non_terminals: &Vec<Token>,
        new_non_terminals: &BTreeSet<BTreeSet<Token>>,
        new_rule_rhs: &mut Vec<BTreeSet<Token>>
    ) {
        if key_rhs.len() == 0 {
            if dict_rules_for_iteration.contains_key(new_rule_rhs) {
                dict_rules_for_iteration.get_mut(new_rule_rhs).unwrap().extend(value_lhs);
            } else {
                dict_rules_for_iteration.insert(new_rule_rhs.clone(), BTreeSet::from_iter(value_lhs.clone().into_iter()));
            }
            return
        }
        let token = key_rhs.get(0).unwrap();
        if non_terminals.contains(&token) {
            for non_term_super_set in new_non_terminals {
                if non_term_super_set.contains(&token) {
                    new_rule_rhs.push(non_term_super_set.clone());
                    Self::add_new_rules(dict_rules_for_iteration, &key_rhs[1..], value_lhs, non_terminals, new_non_terminals, new_rule_rhs);
                    new_rule_rhs.pop();
                    // info!("len {}", new_rule_rhs.len());
                }
            }
        } else {
            new_rule_rhs.push(BTreeSet::from([*token]));
            Self::add_new_rules(dict_rules_for_iteration, &key_rhs[1..], value_lhs, non_terminals, new_non_terminals, new_rule_rhs);
            new_rule_rhs.pop();
            // info!("len {}", new_rule_rhs.len());
        }
    }

    pub fn new(
        mut rules: Vec<Rule>,
        token_types: HashMap<Token, TokenTypes>,
        mut token_raw: HashMap<Token, String>,
        mut tokens_reverse: HashMap<String, (Token, TokenTypes)>,
        mut axiom: Token,
        delim: Token,
    ) -> Result<Grammar, GrammarError> {
        let mut highest_id = delim;
        let mut gen_id = || -> Token {
            highest_id += 1;
            return highest_id;
        };

        let mut inverse_rewrite_rules: HashMap<Token, Vec<Token>> = HashMap::new();
        let mut op_table: HashMap<Token, HashMap<Token, Associativity>> = HashMap::new();

        token_raw.insert(delim, String::from("DELIM"));
        tokens_reverse.insert(String::from("DELIM"), (delim, NonTerminal));

        let mut non_terminals = Vec::new();
        let mut terminals = Vec::new();
        for (id, v) in &token_types {
            if *v == NonTerminal {
                non_terminals.push(*id);
            } else if *v == Terminal {
                terminals.push(*id);
            }
        }

        // Validate that the grammar is in OPG form
        let repeated_rules = Self::get_repeated_rhs(&rules);
        // If we have repeating RHS then we need to do some magic on the grammer to
        // turn it into FNF
        if let Some(repeated_rules) = repeated_rules {
            let new_axiom = gen_id();
            token_raw.insert(new_axiom, String::from("_NewAxiom"));
            tokens_reverse.insert(String::from("_NewAxiom"), (new_axiom, NonTerminal));

            for (rhs, rules) in &repeated_rules {
                warn!("Repeated rhs among the following rules:");
                for r in rules {
                    let mut rhs_formatted = String::new();
                    for t in &r.right {
                        rhs_formatted.push_str(token_raw.get(t).unwrap());
                    }
                    warn!("{} -> {}", token_raw.get(&r.left).unwrap(), rhs_formatted);
                }
            }

            let mut dict_rules: HashMap<Vec<Token>, BTreeSet<Token>> = HashMap::new();
            for r in &rules {
                let mut left = BTreeSet::new();
                left.insert(r.left);
                dict_rules.insert(r.right.clone(), left);
            }

            for (rhs, left) in &dict_rules {
                info!(
                    "dict_rules : {} -> {:?}",
                    token_raw.get(left.iter().next().unwrap()).unwrap(),
                    Self::token_list_to_string(rhs, &token_raw)
                );
            }

            // Delete copy rules
            let mut copy: HashMap<Token, HashSet<Token>> = HashMap::new();
            let mut rhs_dict: HashMap<Token, Vec<Vec<Token>>> = HashMap::new();
            for n in &non_terminals {
                copy.insert(*n, HashSet::new());
            }

            for r in &rules {
                if r.right.len() == 1 {
                    if non_terminals.contains(r.right.get(0).unwrap()) {
                        // It is a copy rule
                        // Update the copy set of rule.left
                        copy.get_mut(&r.left)
                            .unwrap()
                            .insert(*r.right.get(0).unwrap());
                        if dict_rules.contains_key(&r.right) {
                            info!(
                                "Removing : {:?}",
                                Self::token_list_to_string(&r.right, &token_raw)
                            );
                            dict_rules.remove(&r.right).unwrap();
                        }
                    } else {
                        if rhs_dict.contains_key(&r.left) {
                            rhs_dict.get_mut(&r.left).unwrap().push(r.right.clone())
                        } else {
                            rhs_dict.insert(r.left, Vec::from([r.right.clone()]));
                        }
                    }
                }
            }
            let mut changed_copy_sets = true;
            while changed_copy_sets {
                changed_copy_sets = false;
                for n in &non_terminals {
                    let len_copy_set = copy.get(n).unwrap().len();
                    for copy_rhs in copy.get(n).unwrap().clone() {
                        let copy_rhs_hashset = copy.get(&copy_rhs).unwrap().clone();
                        for x in copy_rhs_hashset {
                            copy.get_mut(n).unwrap().insert(x);
                        }
                    }
                    if len_copy_set < copy.get(n).unwrap().len() {
                        changed_copy_sets = true;
                    }
                }
            }
            for n in &non_terminals {
                for copy_rhs in copy.get(n).unwrap() {
                    let empty = Vec::new();
                    let rhs_dict_copy_rhs = rhs_dict.get(copy_rhs).or(Some(&empty)).unwrap();
                    for rhs in rhs_dict_copy_rhs {
                        if !dict_rules.get(rhs).unwrap().contains(n) {
                            let before: Vec<Token> =
                                dict_rules.get(rhs).unwrap().clone().into_iter().collect();
                            let mut other = dict_rules.get(rhs).unwrap().clone();
                            other.insert(*n);
                            dict_rules.get_mut(rhs).unwrap().extend(other);
                            trace!(
                                "Before : {:?} After: {:?}",
                                Self::token_list_to_string(&before, &token_raw),
                                Self::token_list_to_string(
                                    &dict_rules.get(rhs).unwrap().clone().into_iter().collect(),
                                    &token_raw
                                )
                            );
                        }
                    }
                }
            }

            // Professional end to end testing code, nothing to see here...
            // let mut f = File::create("output.txt").unwrap();
            // for (key, val) in dict_rules {
            //     let mut builder = String::new();
            //     builder.push_str("(");
            //     if !key.is_empty() {
            //         builder.push_str(format!("\'{}\'", token_raw.get(&key.get(0).unwrap()).unwrap()).as_str());
            //         if key.len() > 1 {
            //             for k in &key[1..key.len()] {
            //                 builder.push_str(", ");
            //                 builder.push_str(format!("\'{}\'", token_raw.get(&k).unwrap()).as_str());
            //             }
            //         } else {
            //             builder.push_str(",");
            //         }
            //     }
            //     builder.push_str(")\n");
            //     f.write(builder.as_bytes());
            // }

            // Initialize the new nonterminal set V
            let mut V: BTreeSet<BTreeSet<Token>> = dict_rules.clone().into_values().collect();

            let mut new_dict_rules: HashMap<Vec<BTreeSet<Token>>, BTreeSet<Token>> = HashMap::new();
            let mut copied_dict: HashMap<Vec<Token>, BTreeSet<Token>> = HashMap::new();

            // Initialize the new set of productions P with the terminal rules of the original grammar
            // and avoid doing the next checks and expansions for these rules, deleting them from the dictionary of rules
            for (key_rhs, value_lhs) in dict_rules.into_iter() {
                let mut is_terminal_rule = true;
                for t in &key_rhs {
                    if non_terminals.contains(&t) {
                        is_terminal_rule = false;
                        break;
                    }
                }
                if is_terminal_rule {
                    let new_rhs = key_rhs.into_iter().map(|x| BTreeSet::from([x])).collect();
                    new_dict_rules.insert(new_rhs, value_lhs);
                } else {
                    copied_dict.insert(key_rhs, value_lhs);
                }
            }
            let dict_rules = copied_dict;

            // Add the new rules by expanding nonterminals in the rhs
            let mut dict_rules_for_iteration: HashMap<Vec<BTreeSet<Token>>, BTreeSet<Token>> = HashMap::new();
            let mut should_continue = true;
            while should_continue {
                for (key_rhs, value_lhs) in dict_rules.iter() {
                    let mut new_rule_rhs: Vec<BTreeSet<Token>> = Vec::new();
                    Self::add_new_rules(&mut dict_rules_for_iteration, key_rhs, value_lhs, &non_terminals, &mut V, &mut new_rule_rhs);
                }
                let temp  = BTreeSet::from_iter(dict_rules_for_iteration.values().clone().into_iter());
                let mut difference = BTreeSet::new();
                for non_term in temp {
                    if !V.contains(non_term) {
                        difference.insert(non_term.clone());
                    }
                }

                V.append(&mut difference.clone());
                for (key, val) in &dict_rules_for_iteration{
                    new_dict_rules.insert(key.clone(), val.clone());
                }
                if difference.len() == 0 {
                    should_continue = false;
                }
            }

            // List of nonterminals of the invertible grammar G
            let mut V: BTreeSet<BTreeSet<Token>> = new_dict_rules.clone().into_values().collect();

            // This is nonsense, I know. Python can have any value in a set which makes this hard to port
            // without creating this new set.
            let typed_terminals: Vec<BTreeSet<Token>> = terminals.clone().into_iter().map(|x| {BTreeSet::from([x])}).collect();

            // Delete rules with rhs with undefined nonterminals:
            // this implementation of the algorithm can generate rhs of rules with nonterminals which are no more defined.
            //TODO: a bit slightly more efficient version can store beforehand the list of rhs of every nonterminal and then delete the nonterminals whose rhs are all deleted.
            let mut deleted = true;
            while deleted {
                deleted = false;
                new_dict_rules.retain(|key_rhs, value_lhs| {
                    let mut should_keep = true;
                    for token in key_rhs {
                        if (!typed_terminals.contains(token)) && (!V.contains(token)) {
                            deleted = true;
                            should_keep = false;
                            break;
                        }
                    }
                    should_keep
                });
                if deleted {
                    V = new_dict_rules.clone().into_values().collect();
                }
            }

            V.insert(BTreeSet::from([new_axiom]));

            for non_term in &V {
                if non_term.contains(&axiom) {
                    let temp = Vec::from([non_term.clone()]);
                    if non_term.len() == 1 && new_dict_rules.contains_key(&temp) {
                        let entry = new_dict_rules.get_mut(&temp).unwrap().clone();
                        new_dict_rules.insert(Vec::from([BTreeSet::from([new_axiom])]), entry);
                    }
                    new_dict_rules.insert(temp, BTreeSet::from([new_axiom]));
                }
            }

            rules.clear();

            let new_rules = new_dict_rules;
            let new_non_terminal_set = V;

            non_terminals.clear();

            for (rhs, lhs) in new_rules {
                let mut current_rule = Rule::new();

                if lhs.len() == 1 {
                    current_rule.left = *lhs.iter().next().unwrap();
                } else {
                    let mut builder = String::new();
                    let mut iter = lhs.iter();
                    builder.push_str(token_raw.get(iter.next().unwrap()).unwrap());
                    for x in iter {
                        builder.push_str("__");
                        builder.push_str(token_raw.get(x).unwrap())
                    }
                    let new_lhs = gen_id();
                    token_raw.insert(new_lhs, format!("_{}", builder));
                    current_rule.left = new_lhs;
                    tokens_reverse.insert(format!("_{}", builder), (new_lhs, NonTerminal));
                }

                for token in &rhs {
                    if typed_terminals.contains(token) || token.len() == 1 {
                        current_rule.right.push(*token.iter().next().unwrap());
                    } else {
                        let mut builder = String::new();
                        let mut iter = token.iter();
                        builder.push_str(token_raw.get(iter.next().unwrap()).unwrap());
                        for x in iter {
                            builder.push_str("__");
                            builder.push_str(token_raw.get(x).unwrap())
                        }
                        let string = format!("_{}", builder);
                        if let Some((t,_)) = tokens_reverse.get(string.as_str()) {
                            current_rule.right.push(*t);
                        } else {
                            let new_rhs_token = gen_id();
                            token_raw.insert(new_rhs_token, string.clone());
                            tokens_reverse.insert(string, (new_rhs_token, NonTerminal));
                            current_rule.right.push(new_rhs_token);
                        }
                    }
                }
                rules.push(current_rule);
            }

            for n in new_non_terminal_set {
                if n.len() == 1 {
                    non_terminals.push(*n.iter().next().unwrap());
                } else {
                    let mut builder = String::new();
                    let mut iter = n.iter();
                    builder.push_str(token_raw.get(iter.next().unwrap()).unwrap());
                    for x in iter {
                        builder.push_str("__");
                        builder.push_str(token_raw.get(x).unwrap())
                    }
                    let string = format!("_{}", builder);
                    if let Some((t, _)) = tokens_reverse.get(string.as_str()) {
                        non_terminals.push(*t);
                    } else {
                        let new_rhs_token = gen_id();
                        token_raw.insert(new_rhs_token, string.clone());
                        tokens_reverse.insert(string, (new_rhs_token, NonTerminal));
                        non_terminals.push(new_rhs_token);
                    }
                }
            }
            let index = non_terminals.binary_search(&axiom).unwrap();
            non_terminals.remove(index);
            non_terminals.insert(index, new_axiom);
            axiom = new_axiom;

            info!("New Non Terminals");
            for n in &non_terminals {
                info!("{}", token_raw.get(n).unwrap());
            }
        }

        // End of grammar fixing.

        let mut rewrite_rules: HashMap<Token, Vec<Token>> = HashMap::new();
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

        for t in &non_terminals {
            inverse_rewrite_rules.insert(*t, vec![*t]);
        }
        for t in &non_terminals {
            for t1 in rewrite_rules.get(t).unwrap() {
                inverse_rewrite_rules.get_mut(t1).unwrap().push(*t);
            }
        }

        debug!("INVERSE REWRITE RULES");
        let mut largest = 0;
        inverse_rewrite_rules.keys().for_each(|x| {
            let s_len = token_raw.get(x).unwrap().len();
            if s_len > largest {
                largest = s_len
            }
        });
        for row in inverse_rewrite_rules.keys() {
            let row_full_raw: Vec<&String> = inverse_rewrite_rules
                .get(row)
                .unwrap()
                .iter()
                .map(|row_item| token_raw.get(row_item).unwrap())
                .collect();
            debug!(
                "{:s_len$} : {:?}",
                token_raw.get(row).unwrap(),
                row_full_raw,
                s_len = largest
            );
        }

        let mut first_ops: HashMap<Token, HashSet<Token>> = HashMap::new();
        let mut last_ops: HashMap<Token, HashSet<Token>> = HashMap::new();

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
        let mut largest = 0;
        first_ops.keys().for_each(|x| {
            let s_len = token_raw.get(x).unwrap().len();
            if s_len > largest {
                largest = s_len
            }
        });
        for row in first_ops.keys() {
            let row_full_raw: Vec<&String> = first_ops
                .get(row)
                .unwrap()
                .iter()
                .map(|row_item| token_raw.get(row_item).unwrap())
                .collect();
            debug!(
                "{:s_len$} : {:?}",
                token_raw.get(row).unwrap(),
                row_full_raw,
                s_len = largest
            );
        }

        debug!("LAST OP");
        largest = 0;
        last_ops.keys().for_each(|x| {
            let s_len = token_raw.get(x).unwrap().len();
            if s_len > largest {
                largest = s_len
            }
        });
        for row in last_ops.keys() {
            let row_full_raw: Vec<&String> = last_ops
                .get(row)
                .unwrap()
                .iter()
                .map(|row_item| token_raw.get(row_item).unwrap())
                .collect();
            debug!(
                "{:s_len$} : {:?}",
                token_raw.get(row).unwrap(),
                row_full_raw,
                s_len = largest
            );
        }

        let mut template: HashMap<Token, Associativity> = HashMap::new();
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

        // Print op_table
        largest = 0;
        terminals.iter().for_each(|x| {
            let s_len = token_raw.get(x).unwrap().len();
            if s_len > largest {
                largest = s_len
            }
        });
        largest += 1;
        let mut builder = String::new();
        builder.push_str(format!("{:<l$}", "", l = largest).as_str());
        for row in &terminals {
            builder.push_str(format!("{:<l$}", token_raw.get(row).unwrap(), l = largest).as_str());
        }
        debug!("{}", builder);
        builder.clear();

        for row in &terminals {
            builder.push_str(format!("{:<l$}", token_raw.get(row).unwrap(), l = largest).as_str());
            let curr_row = op_table.get(row).unwrap();
            for col in &terminals {
                builder.push_str(
                    format!(
                        "{:<l$}",
                        format!("{:?}", curr_row.get(col).unwrap()),
                        l = largest
                    )
                    .as_str(),
                );
            }
            debug!("{}", builder);
            builder.clear();
        }

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

    pub fn get_precedence(&self, left: Token, right: Token) -> Associativity {
        return self
            .op_table
            .get(&left)
            .unwrap()
            .get(&right)
            .unwrap()
            .clone();
    }
}
