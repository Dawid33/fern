use super::reader::SymbolParserState::InKeyword;
use crate::grammar::error::GrammarError;
use crate::grammar::reader::TokenTypes::{Axiom, NonTerminal, Terminal};
use crate::grammar::{OpGrammar, Rule, Token};
// use crate::{Node, TokenGrammarTuple};
use log::{debug, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::{BTreeSet, HashMap};
use std::error::Error;
use std::fs;
use std::io::{BufReader, Read};
use std::ops::Deref;
use std::prelude::rust_2015;
use std::slice::Iter;

#[derive(Clone, Debug, Copy)]
enum GeneralState {
    ParserSymbols,
    Rules,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
enum SymbolParserState {
    InData,
    InKeyword,
    InIdent,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum RuleParserState {
    InData,
    AwaitingRuleRight,
    InRuleRight,
    InRuleIdentifierRight,
    InRuleLeft,
}

#[derive(Clone, Debug, Copy, PartialEq, Serialize, Deserialize)]
pub enum TokenTypes {
    Terminal,
    Axiom,
    NonTerminal,
    Delim,
}

struct IdCounter {
    highest_id: Token,
}

impl IdCounter {
    pub fn new(start: Token) -> Self {
        Self { highest_id: start }
    }
    pub fn gen_id(&mut self) -> Token {
        self.highest_id += 1;
        return self.highest_id;
    }
}

/// Ad-hoc hand written parser for loading in .g grammar files.
pub struct RawGrammar {
    pub rules: Vec<Rule>,
    pub terminals: Vec<Token>,
    pub non_terminals: Vec<Token>,
    pub token_types: HashMap<Token, TokenTypes>,
    pub token_raw: HashMap<Token, String>,
    pub token_reverse: HashMap<String, (Token, TokenTypes)>,
    pub axiom: Token,
    pub ast_rules: Vec<Rule>,
    pub new_non_terminal_reverse: HashMap<Token, BTreeSet<Token>>,
    pub new_non_terminals_subset: Vec<Token>,
    pub reduction_tree: ReductionTree,
    pub foobar: HashMap<Token, ReductionTree>,
    pub old_axiom: Token,
    id_counter: IdCounter,
}

impl RawGrammar {
    pub fn from(path: &str, lexical_sync: Vec<String>) -> Result<Self, Box<dyn Error>> {
        let mut file = fs::File::open(path).unwrap();
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        Ok(RawGrammar::new(buf.as_str(), lexical_sync)?)
    }
    pub fn new(s: &str, lexical_sync: Vec<String>) -> Result<RawGrammar, GrammarError> {
        info!("{:?}", lexical_sync);
        let mut state = GeneralState::ParserSymbols;
        let mut symbol_parser_state = SymbolParserState::InData;
        let mut rule_parser_state = RuleParserState::InData;
        let mut previous: char = 0 as char;
        let mut buf = String::new();
        let mut nesting_buf = String::new();
        let mut awaiting: Option<TokenTypes> = None;
        let mut token_reverse: HashMap<String, (Token, TokenTypes)> = HashMap::new();
        let mut axiom: Option<Token> = None;
        let mut id_counter = IdCounter::new(lexical_sync.len() - 1);

        let mut rules: Vec<Rule> = Vec::new();
        let mut rule: Option<Rule> = None;

        for c in s.chars() {
            match state {
                GeneralState::ParserSymbols => match c {
                    '%' => {
                        if previous == '%' {
                            state = GeneralState::Rules;
                            continue;
                        } else if let None = awaiting {
                            symbol_parser_state = SymbolParserState::InKeyword;
                        }
                    }
                    ' ' | '\n' | '\t' => {
                        match symbol_parser_state {
                            SymbolParserState::InKeyword => {
                                if buf.eq("terminal") {
                                    awaiting = Some(Terminal);
                                } else if buf.eq("nonterminal") {
                                    awaiting = Some(NonTerminal);
                                } else if buf.eq("axiom") {
                                    awaiting = Some(Axiom);
                                } else {
                                    return Err(GrammarError::from(format!("Invalid keyword : {}", buf.as_str())));
                                }
                                buf.clear();
                            }
                            SymbolParserState::InIdent => {
                                if let Some(t) = awaiting {
                                    match t {
                                        Terminal => {
                                            if lexical_sync.contains(&buf) {
                                                let i = lexical_sync.iter().position(|x| x == &buf).unwrap();
                                                token_reverse.insert(buf.clone(), (i, Terminal));
                                            } else {
                                                token_reverse.insert(buf.clone(), (id_counter.gen_id(), Terminal));
                                            }
                                        }
                                        Axiom => axiom = Some(token_reverse.get(buf.as_str()).unwrap().0),
                                        NonTerminal => {
                                            token_reverse.insert(buf.clone(), (id_counter.gen_id(), NonTerminal));
                                        }
                                        _ => {}
                                    }
                                    awaiting = None;
                                    buf.clear();
                                } else {
                                    return Err(GrammarError::from(format!("Rogue identifier : {}", buf.as_str())));
                                }
                            }
                            SymbolParserState::InData => (),
                        }
                        symbol_parser_state = SymbolParserState::InData;
                    }
                    'A'..='Z' | 'a'..='z' | '0'..='9' | '_' => {
                        if symbol_parser_state == SymbolParserState::InData {
                            symbol_parser_state = SymbolParserState::InIdent;
                        }
                        buf.push(c);
                    }
                    _ => {
                        return Err(GrammarError::from(format!("Invalid character in grammar definition: {}", c)));
                    }
                },
                GeneralState::Rules => match c {
                    ' ' | '\t' => match rule_parser_state {
                        RuleParserState::InRuleLeft => {
                            let (id, _) = token_reverse.get(&*buf).unwrap();
                            rule = Some(Rule::from(*id));
                            rule_parser_state = RuleParserState::AwaitingRuleRight;
                        }
                        RuleParserState::InRuleIdentifierRight => {
                            let (id, _) = token_reverse.get(&*buf).unwrap();
                            rule.as_mut().unwrap().right.push(*id);
                            rule_parser_state = RuleParserState::InRuleRight;
                            let mut b = String::new();
                            let mut nesting: Vec<i16> = Vec::new();
                            if !nesting_buf.is_empty() {
                                for c in nesting_buf.chars() {
                                    if c == '.' {
                                        if !b.is_empty() {
                                            nesting.push(b.parse().unwrap());
                                        }
                                        b.clear();
                                    } else {
                                        b.push(c);
                                    }
                                }
                                if !b.is_empty() {
                                    nesting.push(b.parse().unwrap());
                                }
                            } else {
                                nesting.push(-1);
                            }
                            nesting_buf.clear();
                            rule.as_mut().unwrap().nesting_rules.push(nesting);
                        }
                        RuleParserState::InRuleRight | RuleParserState::AwaitingRuleRight | RuleParserState::InData => (),
                    },
                    ':' | '|' => match rule_parser_state {
                        RuleParserState::InData => {
                            return Err(GrammarError::from("Identifier should precede :.".to_string()));
                        }
                        RuleParserState::InRuleLeft | RuleParserState::AwaitingRuleRight => {
                            rule_parser_state = RuleParserState::InRuleRight;
                            rule.as_mut().unwrap().right.clear();
                            rule.as_mut().unwrap().nesting_rules.clear();
                            nesting_buf.clear();
                        }
                        RuleParserState::InRuleRight => {
                            return Err(GrammarError::from("Illegal char : in right rule".to_string()));
                        }
                        RuleParserState::InRuleIdentifierRight => {
                            return Err(GrammarError::from("Illegal char : in right rule".to_string()));
                        }
                    },
                    '\n' => match rule_parser_state {
                        RuleParserState::InData | RuleParserState::AwaitingRuleRight => (),
                        RuleParserState::InRuleRight => {
                            rule_parser_state = RuleParserState::AwaitingRuleRight;
                            if let Some(r) = rule.clone() {
                                rules.push(r.clone());
                            } else {
                                return Err(GrammarError::from("Semicolon used without rule preceding it.".to_string()));
                            }
                        }
                        RuleParserState::InRuleIdentifierRight => {
                            rule_parser_state = RuleParserState::AwaitingRuleRight;
                            let (id, _) = token_reverse.get(&*buf).unwrap();
                            rule.as_mut().unwrap().right.push(*id);
                            let mut b = String::new();
                            let mut nesting: Vec<i16> = Vec::new();
                            if !nesting_buf.is_empty() {
                                for c in nesting_buf.chars() {
                                    if c == '.' {
                                        if !b.is_empty() {
                                            nesting.push(b.parse().unwrap());
                                        }
                                        b.clear();
                                    } else {
                                        b.push(c);
                                    }
                                }
                                if !b.is_empty() {
                                    nesting.push(b.parse().unwrap());
                                }
                            } else {
                                nesting.push(-1);
                            }
                            nesting_buf.clear();
                            rule.as_mut().unwrap().nesting_rules.push(nesting);
                            rules.push(rule.as_mut().unwrap().clone());
                        }
                        RuleParserState::InRuleLeft => {
                            return Err(GrammarError::from("Unexected new line after left rule.".to_string()));
                        }
                    },
                    ';' => {
                        rule_parser_state = RuleParserState::InData;
                        rule = None;
                    }
                    'A'..='Z' | 'a'..='z' | '0'..='9' | '_' | '.' => match rule_parser_state {
                        RuleParserState::InData => {
                            rule_parser_state = RuleParserState::InRuleLeft;
                            buf.clear();
                            buf.push(c);
                        }
                        RuleParserState::InRuleRight => {
                            rule_parser_state = RuleParserState::InRuleIdentifierRight;
                            buf.clear();
                            buf.push(c);
                        }
                        RuleParserState::InRuleLeft => buf.push(c),
                        RuleParserState::InRuleIdentifierRight => match c {
                            'A'..='Z' | 'a'..='z' => buf.push(c),
                            '0'..='9' | '_' | '.' => {
                                nesting_buf.push(c);
                            }
                            _ => {
                                panic!("Shouldn't happen")
                            }
                        },
                        RuleParserState::AwaitingRuleRight => {
                            return Err(GrammarError::from("Expected :, | or ;, found start of identifier.".to_string()));
                        }
                    },
                    _ => {
                        return Err(GrammarError::from(format!("Invalid character in grammar definition: {}", c)));
                    }
                },
            }
            previous = c;
        }

        let mut token_types: HashMap<Token, TokenTypes> = HashMap::new();
        let mut token_raw: HashMap<Token, String> = HashMap::new();
        for (raw, (id, token_type)) in &token_reverse {
            token_types.insert(*id, *token_type);
            token_raw.insert(*id, raw.clone());
        }
        let axiom: Token = axiom.expect("Need to specify and axiom.");

        let mut ast_rules = Vec::new();
        for r in &rules {
            'outer: for x in &r.nesting_rules {
                for y in x {
                    if *y != -1 {
                        ast_rules.push(r.clone());
                        break 'outer;
                    }
                }
            }
        }

        let mut foobar: HashMap<Token, ReductionTree> = HashMap::new();

        let mut r_tree = ReductionTree::new();
        for r in &rules {
            r_tree.add_rule(r);
            if foobar.contains_key(&r.left) {
                foobar.get_mut(&r.left).unwrap().add_rule(r);
            } else {
                let mut new_r_tree = ReductionTree::new();
                new_r_tree.add_rule(r);
                foobar.insert(r.left, new_r_tree);
            }
            let mut output = Vec::new();
            for (i, t) in r.right.iter().enumerate() {
                output.push(format!("({}, {:?}), ", token_raw.get(t).unwrap().clone(), r.nesting_rules.get(i).unwrap()));
            }
            trace!("Rule : {} -> {:?}", &token_raw.get(&r.left).unwrap(), output,);
        }

        let mut non_terminals = Vec::new();
        let mut terminals = Vec::new();
        for (id, v) in &token_types {
            if *v == NonTerminal {
                non_terminals.push(*id);
            } else if *v == Terminal {
                terminals.push(*id);
            }
        }

        Ok(RawGrammar {
            rules,
            terminals,
            non_terminals,
            token_types,
            token_raw,
            token_reverse,
            axiom,
            id_counter,
            ast_rules,
            new_non_terminal_reverse: HashMap::new(),
            new_non_terminals_subset: Vec::new(),
            reduction_tree: r_tree,
            foobar,
            old_axiom: axiom,
        })
    }
    pub fn gen_id(&mut self) -> Token {
        self.id_counter.gen_id()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReductionTree {
    root_nodes: HashMap<Token, Vec<ReductionNode>>,
}

impl ReductionTree {
    pub fn new() -> Self {
        Self { root_nodes: HashMap::new() }
    }

    // pub fn disambiguate<T>(&self, node: &Node<T>, g: &OpGrammar, _expected: Option<Token>) -> Vec<&Rule> {
    //     let mut output: Vec<&Rule> = Vec::new();
    //     let mut iter = node.children.iter();
    //     let first = iter.next().unwrap();

    //     let current: Option<_> = if g.new_non_terminal_reverse.contains_key(&first.symbol) {
    //         let mut result = None;
    //         for t in g.new_non_terminal_reverse.get(&first.symbol).unwrap() {
    //             if let Some(next_nodes) = self.root_nodes.get(t) {
    //                 debug!("Disambiguate - Matched : {}", g.token_raw.get(t).unwrap());
    //                 result = Some(next_nodes);
    //                 break;
    //             }
    //         }
    //         result
    //     } else {
    //         if let Some(next_nodes) = self.root_nodes.get(&first.symbol) {
    //             debug!("Disambiguate - Matched : {}", g.token_raw.get(&first.symbol).unwrap());
    //             Some(next_nodes)
    //         } else {
    //             None
    //         }
    //     };

    //     if let Some(mut current) = current {
    //         for child in iter {
    //             let possible_options_for_child: Vec<Token> = if let Some(t) = g.new_non_terminal_reverse.get(&child.symbol) {
    //                 t.iter().map(|x| *x).rev().collect()
    //             } else {
    //                 Vec::from([child.symbol])
    //             };

    //             let mut has_changed = false;
    //             for t in possible_options_for_child {
    //                 let mut found_node: Option<usize> = None;
    //                 for (i, exiting_node) in current.iter().enumerate() {
    //                     match exiting_node {
    //                         ReductionNode::Node(n, _) => {
    //                             if *n == t {
    //                                 found_node = Some(i);
    //                                 break;
    //                             }
    //                         }
    //                         ReductionNode::Rule(_) => (),
    //                     }
    //                 }
    //                 current = if let Some(i) = found_node {
    //                     match current.get(i).unwrap() {
    //                         ReductionNode::Node(t, vec) => {
    //                             debug!("Disambiguate - Matched : {}", g.token_raw.get(t).unwrap());
    //                             has_changed = true;
    //                             vec
    //                         }
    //                         ReductionNode::Rule(_) => {
    //                             unreachable!()
    //                         }
    //                     }
    //                 } else {
    //                     current
    //                 };
    //                 if has_changed {
    //                     break;
    //                 }
    //             }
    //         }

    //         for (_i, exiting_node) in current.iter().enumerate() {
    //             match exiting_node {
    //                 ReductionNode::Rule(r) => {
    //                     debug!("Found : {:?}", r);
    //                     output.push(r);
    //                 }
    //                 _ => (),
    //             }
    //         }
    //     }
    //     for r in &output {
    //         info!("Could be {}", g.token_raw.get(&r.left).unwrap());
    //     }
    //     output
    // }

    // let mut iter = rhs.iter();
    // let first = iter.next().unwrap();
    // let mut current = if let Some(next_nodes) = self.root_nodes.get(&first) {
    //     debug!("Get Rules - Matched : {}", tokens_raw.get(&first).unwrap());
    //     next_nodes
    // } else {
    //     return None;
    // };
    //
    // for t in iter {
    //     let mut found_node: Option<usize> = None;
    //     for (i, exiting_node) in current.iter().enumerate() {
    //         match exiting_node {
    //             ReductionNode::Node(n, _) => {
    //                 if *n == *t {
    //                     found_node = Some(i);
    //                     break;
    //                 }
    //             },
    //             ReductionNode::Rule(_) => {
    //                 if found_node.is_none() {
    //                     found_node = Some(i);
    //                 }
    //             }
    //         }
    //     }
    //     current = if let Some(i) = found_node {
    //         match current.get(i).unwrap() {
    //             ReductionNode::Node(t, vec) => {
    //                 debug!("Matched : {}", tokens_raw.get(t).unwrap());
    //                 vec
    //             },
    //             ReductionNode::Rule(r) => {
    //                 debug!("Found : {:?}", r);
    //                 return Some(r);
    //             }
    //         }
    //     } else {
    //         return None;
    //     };
    // }
    //
    // for (i, exiting_node) in current.iter().enumerate() {
    //     match exiting_node {
    //         ReductionNode::Rule(r) => {
    //             debug!("Found : {:?}", r);
    //             return Some(r)
    //         },
    //         _ => (),
    //     }
    // }

    pub fn match_rule(&self, rhs: &[&Token], tokens_raw: &HashMap<Token, String>) -> Option<&Rule> {
        if rhs.len() == 0 {
            return None;
        }
        let mut iter = rhs.iter();
        let first = iter.next().unwrap();
        let mut current = if let Some(next_nodes) = self.root_nodes.get(&first) {
            debug!("Matched : {}", tokens_raw.get(&first).unwrap());
            next_nodes
        } else {
            return None;
        };

        for t in iter {
            let mut found_node: Option<usize> = None;
            for (i, exiting_node) in current.iter().enumerate() {
                match exiting_node {
                    ReductionNode::Node(n, _) => {
                        if *n == **t {
                            found_node = Some(i);
                            break;
                        }
                    }
                    ReductionNode::Rule(_) => {
                        if found_node.is_none() {
                            found_node = Some(i);
                        }
                    }
                }
            }
            current = if let Some(i) = found_node {
                match current.get(i).unwrap() {
                    ReductionNode::Node(t, vec) => {
                        debug!("Matched : {}", tokens_raw.get(t).unwrap());
                        vec
                    }
                    ReductionNode::Rule(r) => {
                        debug!("Found : {:?}", r);
                        return Some(r);
                    }
                }
            } else {
                return None;
            };
        }

        for (_i, exiting_node) in current.iter().enumerate() {
            match exiting_node {
                ReductionNode::Rule(r) => {
                    debug!("Found : {:?}", r);
                    return Some(r);
                }
                _ => (),
            }
        }
        None
    }

    pub fn add_rule(&mut self, r: &Rule) {
        if r.right.len() == 0 {
            return;
        }
        let first = r.right.first().unwrap();
        if !self.root_nodes.contains_key(first) {
            self.root_nodes.insert(*first, vec![ReductionNode::Node(*first, Vec::new())]);
        }

        let mut iter = r.right.iter();
        let current = self.root_nodes.get_mut(iter.next().unwrap()).unwrap();
        Self::build_next(iter, current, r);
    }

    fn build_next(mut new_term_iter: Iter<Token>, current: &mut Vec<ReductionNode>, r: &Rule) {
        let next_term = new_term_iter.next();
        if let Some(t) = next_term {
            let mut found_node: Option<usize> = None;
            for (i, exiting_node) in current.iter().enumerate() {
                match exiting_node {
                    ReductionNode::Node(n, _vec) => {
                        if *n == *t {
                            found_node = Some(i);
                            break;
                        }
                    }
                    ReductionNode::Rule(_) => {}
                }
            }
            let next = if let Some(i) = found_node {
                current.get_mut(i).unwrap()
            } else {
                current.push(ReductionNode::Node(*t, Vec::new()));
                current.last_mut().unwrap()
            };
            if let ReductionNode::Node(_, vec) = next {
                Self::build_next(new_term_iter, vec, r);
            } else {
                unreachable!();
            }
        } else {
            current.push(ReductionNode::Rule(r.clone()));
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ReductionNode {
    Node(Token, Vec<ReductionNode>),
    Rule(Rule),
}
