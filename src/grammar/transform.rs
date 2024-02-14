use super::opg::RawGrammar;
use super::GrammarError;
use crate::grammar::lg::Token;
use crate::grammar::opg::{OpGrammar, Rule, TokenTypes};
use log::{debug, info, trace, warn};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::Write;

impl RawGrammar {
    pub fn delete_repeated_rhs(&mut self) -> Result<(), GrammarError> {
        // let repeated_rules = if let Some(repeated_rules) = self.get_repeated_rhs() {
        //     repeated_rules
        // } else {
        //     return Err(GrammarError::from("Cannot delete repeated rules as there are no repeated rules.".to_string()));
        // };
        let repeated_rules = self.get_repeated_rhs().unwrap_or(HashMap::new());

        let new_axiom = self.gen_id();
        self.token_raw.insert(new_axiom, String::from("_NewAxiom"));
        self.token_reverse.insert(String::from("_NewAxiom"), (new_axiom, TokenTypes::NonTerminal));
        for (_, rules) in &repeated_rules {
            trace!("Repeated rhs among the following rules:");
            for r in rules {
                let mut rhs_formatted = String::new();
                for t in &r.right {
                    rhs_formatted.push_str(self.token_raw.get(t).unwrap());
                }
                trace!("{} -> {}", self.token_raw.get(&r.left).unwrap(), rhs_formatted);
            }
        }

        let mut dict_rules: HashMap<Vec<Token>, BTreeSet<Token>> = HashMap::new();
        for r in &self.rules {
            let mut left = BTreeSet::new();
            left.insert(r.left);
            if dict_rules.contains_key(&r.right) {
                dict_rules.get_mut(&r.right).unwrap().extend([r.left]);
            } else {
                dict_rules.insert(r.right.clone(), left);
            }
        }

        for (rhs, left) in &dict_rules {
            trace!(
                "dict_rules : {} -> {:?}",
                self.token_raw.get(left.iter().next().unwrap()).unwrap(),
                OpGrammar::token_list_to_string(rhs, &self.token_raw)
            );
        }

        trace!("Direct ambiguities :");
        for (rhs, lhs) in &dict_rules {
            if lhs.len() > 1 {
                let mut b = String::new();
                for t in rhs {
                    b.push_str(format!("({} ", self.token_raw.get(&t).unwrap()).as_str());
                }
                b.push_str(") might be one of (");
                for t in lhs {
                    b.push_str(format!("{} ", self.token_raw.get(&t).unwrap()).as_str());
                }
                trace!("{})", b);
            }
        }

        // Delete copy rules
        trace!("Deleting copy rules");
        let mut copy: HashMap<Token, HashSet<Token>> = HashMap::new();
        let mut rhs_dict: HashMap<Token, Vec<Vec<Token>>> = HashMap::new();
        for n in &self.non_terminals {
            copy.insert(*n, HashSet::new());
        }

        for r in &self.rules {
            if r.right.len() == 1 && self.non_terminals.contains(r.right.get(0).unwrap()) {
                // It is a copy rule
                // Update the copy set of rule.left
                let old = copy.get_mut(&r.left).unwrap().clone(); // Unused
                copy.get_mut(&r.left).unwrap().insert(r.right.get(0).unwrap().clone());
                trace!(
                    "Update: {:?} -> {:?}",
                    OpGrammar::token_list_to_string(&old.into_iter().collect(), &self.token_raw),
                    OpGrammar::token_list_to_string(&copy.get(&r.left).unwrap().clone().into_iter().collect(), &self.token_raw)
                );
                if dict_rules.contains_key(&r.right) {
                    trace!("Removing : {:?}", OpGrammar::token_list_to_string(&r.right, &self.token_raw));
                    dict_rules.remove(&r.right).unwrap();
                }
            } else {
                if rhs_dict.contains_key(&r.left) {
                    trace!("Pushing: {:?}", OpGrammar::token_list_to_string(&r.right, &self.token_raw));
                    rhs_dict.get_mut(&r.left).unwrap().push(r.right.clone());
                } else {
                    trace!(
                        "Inserting : {:?} -> {:?}",
                        self.token_raw.get(&r.left).unwrap(),
                        OpGrammar::token_list_to_string(&r.right, &self.token_raw)
                    );
                    rhs_dict.insert(r.left, Vec::from([r.right.clone()]));
                }
            }
        }
        let mut changed_copy_sets = true;
        while changed_copy_sets {
            changed_copy_sets = false;
            for n in &self.non_terminals {
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

        // let mut f = File::create("copy.txt").unwrap();
        // for (key, val) in &copy {
        //     let mut builder = String::new();
        //     builder.push_str(format!("{} = [", self.token_raw.get(&key).unwrap()).as_str());
        //
        //     let mut sorted = Vec::new();
        //     for x in val.iter() {
        //         sorted.push(self.token_raw.get(x).unwrap());
        //     }
        //     sorted.sort();
        //
        //     let mut val_iter = sorted.iter();
        //     if val_iter.len() > 0 {
        //         builder.push_str(format!("\'{}\'", val_iter.next().unwrap()).as_str());
        //     }
        //     while let Some(t) = val_iter.next() {
        //         builder.push_str(", ");
        //         builder.push_str(format!("\'{}\'", t).as_str());
        //     }
        //     builder.push_str("]\n");
        //     f.write(builder.as_bytes()).unwrap();
        // }

        for n in &self.non_terminals {
            for copy_rhs in copy.get(n).unwrap() {
                let empty = Vec::new();
                let rhs_dict_copy_rhs = rhs_dict.get(copy_rhs).or(Some(&empty)).unwrap();
                for rhs in rhs_dict_copy_rhs {
                    if !dict_rules.get(rhs).unwrap().contains(n) {
                        dict_rules.get_mut(rhs).unwrap().extend([n]);
                    }
                }
            }
        }

        // Initialize the new nonterminal set V
        // print_dict("should_be_concated.txt", &dict_rules, &self.token_raw);
        trace!("Init new nonterminal set V");
        let temp = dict_rules.clone().into_values();
        let mut v: BTreeSet<BTreeSet<Token>> = BTreeSet::new();
        for x in temp {
            v.insert(x);
        }

        let mut new_dict_rules: HashMap<Vec<Vec<Token>>, BTreeSet<Token>> = HashMap::new();
        let mut copied_dict: HashMap<Vec<Token>, BTreeSet<Token>> = HashMap::new();

        // Initialize the new set of productions P with the terminal rules of the original grammar
        // and avoid doing the next checks and expansions for these rules, deleting them from the
        // dictionary of rules
        for (key_rhs, value_lhs) in dict_rules.into_iter() {
            let mut is_terminal_rule = true;
            for t in &key_rhs {
                if self.non_terminals.contains(&t) {
                    is_terminal_rule = false;
                    break;
                }
            }
            if is_terminal_rule {
                new_dict_rules.insert(vec![key_rhs.clone()], value_lhs);
            } else {
                copied_dict.insert(key_rhs, value_lhs);
            }
        }
        let dict_rules = copied_dict;

        // let mut f = File::create("V.txt").unwrap();
        //  for val in &v {
        //      let mut builder = String::new();
        //      builder.push_str("[");
        //
        //      let mut val_iter = val.iter();
        //      if let Some(t) = val_iter.next() {
        //          builder.push_str(format!("\'{}\'", self.token_raw.get(t).unwrap()).as_str());
        //      }
        //      while let Some(t) = val_iter.next() {
        //          builder.push_str(", ");
        //          builder.push_str(format!("\'{}\'", self.token_raw.get(t).unwrap()).as_str());
        //      }
        //      builder.push_str("]\n");
        //      f.write(builder.as_bytes());
        //  }
        let mut non_terms_chunked: HashMap<BTreeSet<Token>, Vec<BTreeSet<Token>>> = HashMap::new();
        // for x in &v {
        //     non_terms_chunked.insert(x.clone(), vec![x.clone()]);
        // }

        // Add the new rules by expanding nonterminals in the rhs
        trace!("big scary dict recursive part");
        let mut dict_rules_for_iteration: HashMap<Vec<Vec<Token>>, BTreeSet<BTreeSet<Token>>> = HashMap::new();
        let mut recursive_part = || {
            let mut should_continue: bool = true;
            while should_continue {
                for (key_rhs, value_lhs) in dict_rules.iter() {
                    let mut new_rule_rhs: Vec<Vec<Token>> = Vec::new();
                    Self::add_new_rules(
                        &mut dict_rules_for_iteration,
                        key_rhs,
                        value_lhs,
                        &self.non_terminals,
                        &mut v,
                        &mut new_rule_rhs,
                        &self.token_raw,
                        &self.token_reverse,
                    );
                }
                let temp = BTreeSet::from_iter(dict_rules_for_iteration.values().clone().into_iter());
                let mut difference = BTreeSet::new();
                for new_non_term_chunked in temp {
                    let mut non_term = BTreeSet::new();
                    for x in new_non_term_chunked {
                        non_term.extend(x);
                    }
                    if !v.contains(&non_term) {
                        non_terms_chunked.insert(
                            non_term.clone(),
                            Vec::from(new_non_term_chunked.clone().into_iter().collect::<Vec<BTreeSet<Token>>>()),
                        );
                        difference.insert(non_term);
                    }
                }

                v.extend(difference.clone().into_iter());
                for (key, val) in &dict_rules_for_iteration {
                    let mut result = Vec::new();
                    for set in key {
                        result.push(set.clone().into_iter().collect());
                    }

                    let mut non_term = BTreeSet::new();
                    for x in val {
                        non_term.extend(x);
                    }
                    new_dict_rules.insert(result, non_term);
                }
                if difference.len() == 0 {
                    should_continue = false;
                }
            }
        };

        recursive_part();

        // List of nonterminals of the invertible grammar G
        let mut v: BTreeSet<BTreeSet<Token>> = new_dict_rules.clone().into_values().collect();

        // Delete rules with rhs with undefined nonterminals:
        // this implementation of the algorithm can generate rhs of rules with nonterminals which are
        // no more defined.
        //TODO: a bit slightly more efficient version can store beforehand the list of rhs of every
        // nonterminal and then delete the nonterminals whose rhs are all deleted.
        let mut deleted = true;
        trace!("finished big scary part");
        while deleted {
            deleted = false;
            new_dict_rules.retain(|key_rhs, _| {
                let mut should_keep = true;
                for vec_token in key_rhs {
                    let token: BTreeSet<Token> = vec_token.clone().into_iter().collect();
                    let mut is_terminal = false;
                    for x in &token {
                        if self.terminals.contains(&x) {
                            is_terminal = true;
                            break;
                        }
                    }
                    if (!is_terminal) && (!v.contains(&token)) {
                        deleted = true;
                        should_keep = false;
                        break;
                    }
                }
                should_keep
            });
            if deleted {
                v = new_dict_rules.clone().into_values().collect();
            }
        }

        v.insert(BTreeSet::from([new_axiom]));

        //Add rules for the axiom of G, which have as rhs all new nonterminals that contain the old axiom
        for non_term in &v {
            if non_term.contains(&self.axiom) {
                let temp = Vec::from([non_term.clone().into_iter().collect()]);
                // If the rule has exactly the old axiom as rhs, replace it with the new axiom
                if non_term.len() == 1 && new_dict_rules.contains_key(&temp) {
                    let entry = new_dict_rules.get_mut(&temp).unwrap().clone();
                    new_dict_rules.insert(Vec::from([Vec::from([new_axiom])]), entry);
                }
                new_dict_rules.insert(temp, BTreeSet::from([new_axiom]));
            }
        }

        // let mut f = File::create("non_terms_chunked.txt").unwrap();
        // for (key, val) in &non_terms_chunked {
        //     let mut builder = String::new();
        //     builder.push_str("[");
        //     let mut key_iter = key.iter();
        //     if let Some(t) = key_iter.next() {
        //         builder.push_str(format!("\'{}\'", self.token_raw.get(t).unwrap()).as_str());
        //     }
        //     while let Some(t) = key_iter.next() {
        //         builder.push_str(", ");
        //         builder.push_str(format!("\'{}\'", self.token_raw.get(t).unwrap()).as_str());
        //     }
        //     builder.push_str("] = [");
        //     if !val.is_empty() {
        //         let mut val_iter = val.get(0).unwrap().iter();
        //         if let Some(t) = val_iter.next() {
        //             builder.push_str(format!("\'{}\'", self.token_raw.get(t).unwrap()).as_str());
        //         }
        //         while let Some(t) = val_iter.next() {
        //             builder.push_str(", ");
        //             builder.push_str(format!("\'{}\'", self.token_raw.get(t).unwrap()).as_str());
        //         }
        //         if val.len() > 1 {
        //             for k in &val[1..val.len()] {
        //                 builder.push_str("], [");
        //                 let mut val_iter = val.get(0).unwrap().iter();
        //                 if let Some(t) = val_iter.next() {
        //                     builder.push_str(format!("\'{}\'", self.token_raw.get(t).unwrap()).as_str());
        //                 }
        //                 while let Some(t) = val_iter.next() {
        //                     builder.push_str(", ");
        //                     builder.push_str(format!("\'{}\'", self.token_raw.get(t).unwrap()).as_str());
        //                 }
        //             }
        //         }
        //     }
        //     builder.push_str("]\n");
        //     f.write(builder.as_bytes());
        // }

        self.rules.clear();
        self.non_terminals.clear();
        let new_rules = new_dict_rules;
        let new_non_terminal_set = v;

        for n in new_non_terminal_set {
            let cloned = n.clone();
            let n = Vec::from_iter(n.into_iter());
            if n.len() == 1 {
                self.non_terminals.push(*n.get(0).unwrap());
            } else {
                let joined = OpGrammar::list_to_string(&n, &self.token_raw);
                if let Some((t, _)) = self.token_reverse.get(joined.as_str()) {
                    self.non_terminals.push(*t);
                } else {
                    let new_rhs_token = self.gen_id();
                    self.token_raw.insert(new_rhs_token, joined.clone());
                    self.token_reverse.insert(joined, (new_rhs_token, TokenTypes::NonTerminal));
                    self.non_terminals.push(new_rhs_token);
                    self.new_non_terminal_reverse.insert(new_rhs_token, cloned);
                    self.new_non_terminals_subset.push(new_rhs_token);
                }
            }
        }

        for (rhs, lhs) in new_rules {
            let lhs = Vec::from_iter(lhs.into_iter());
            let mut current_rule = Rule::new();

            if lhs.len() == 1 {
                current_rule.left = *lhs.get(0).unwrap();
            } else {
                let joined = OpGrammar::list_to_string(&lhs, &self.token_raw);
                if let Some((t, _)) = self.token_reverse.get(joined.as_str()) {
                    current_rule.left = *t;
                } else {
                    panic!("Token '{}' does not exist.", joined);
                }
            }

            for mut token in rhs {
                let mut is_terminal = false;
                for x in &token {
                    if self.terminals.contains(x) || token.len() == 1 {
                        is_terminal = true;
                        break;
                    }
                }
                if is_terminal {
                    current_rule.right.append(&mut token);
                } else {
                    let joined = OpGrammar::list_to_string(&token, &self.token_raw);
                    if let Some((t, _)) = self.token_reverse.get(joined.as_str()) {
                        current_rule.right.push(*t);
                    } else {
                        panic!("Token '{}' does not exist.", joined);
                    }
                }
            }
            self.rules.push(current_rule);
        }

        trace!("{} New Non Terminals and {} rules", self.non_terminals.len(), self.rules.len());
        for n in &self.non_terminals {
            trace!("{}", self.token_raw.get(n).unwrap());
        }

        self.axiom = new_axiom;
        Ok(())
    }

    fn add_new_rules(
        dict_rules_for_iteration: &mut HashMap<Vec<Vec<Token>>, BTreeSet<BTreeSet<Token>>>,
        key_rhs: &[Token],
        value_lhs: &BTreeSet<Token>,
        non_terminals: &Vec<Token>,
        new_non_terminals: &BTreeSet<BTreeSet<Token>>,
        new_rule_rhs: &mut Vec<Vec<Token>>,
        token_raw: &BTreeMap<Token, String>,
        token_reverse: &BTreeMap<String, (Token, TokenTypes)>,
    ) {
        if key_rhs.len() == 0 {
            if dict_rules_for_iteration.contains_key(new_rule_rhs) {
                dict_rules_for_iteration.get_mut(new_rule_rhs).unwrap().insert(value_lhs.clone());
            } else {
                dict_rules_for_iteration.insert(new_rule_rhs.clone(), BTreeSet::from([value_lhs.clone()]));
            }
            return;
        }
        let token = key_rhs.get(0).unwrap();
        if non_terminals.contains(&token) {
            for non_term_super_set in new_non_terminals {
                if non_term_super_set.contains(&token) {
                    new_rule_rhs.push(non_term_super_set.clone().into_iter().collect());
                    Self::add_new_rules(
                        dict_rules_for_iteration,
                        &key_rhs[1..],
                        value_lhs,
                        non_terminals,
                        new_non_terminals,
                        new_rule_rhs,
                        token_raw,
                        token_reverse,
                    );
                    new_rule_rhs.pop();
                }
            }
        } else {
            new_rule_rhs.push(Vec::from([*token]));
            Self::add_new_rules(
                dict_rules_for_iteration,
                &key_rhs[1..],
                value_lhs,
                non_terminals,
                new_non_terminals,
                new_rule_rhs,
                token_raw,
                token_reverse,
            );
            new_rule_rhs.pop();
        }
    }

    pub fn get_repeated_rhs(&mut self) -> Option<HashMap<Vec<Token>, Vec<Rule>>> {
        let mut repeated_rules: HashMap<Vec<Token>, Vec<Rule>> = HashMap::new();
        let mut rhs_rule_map: HashMap<Vec<Token>, Vec<Rule>> = HashMap::new();
        for r in &self.rules {
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
}
