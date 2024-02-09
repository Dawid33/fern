use super::transform::*;
use super::GrammarError;
use crate::grammar::print_op_table;
use crate::grammar::Associativity::{Equal, Left, Right};
use log::{debug, info, trace};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::hash_map::HashMap;
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::error::Error;
use std::fmt::{format, Debug};
use std::fs;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufReader, Read};
use std::io::{Seek, Write};
use std::ops::Deref;
use std::prelude::rust_2015;
use std::slice::Iter;

pub type Token = usize;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Rule {
    pub left: Token,
    pub right: Vec<Token>,
    pub nesting_rules: Vec<Vec<i16>>,
}

/// Ad-hoc hand written parser for loading in .g grammar files.
pub struct RawGrammar {
    pub rules: Vec<Rule>,
    pub token_map: Vec<String>,
    pub terminals: Vec<Token>,
    pub non_terminals: Vec<Token>,
    pub token_types: HashMap<Token, TokenTypes>,
    pub token_raw: HashMap<Token, String>,
    pub token_reverse: BTreeMap<String, (Token, TokenTypes)>,
    pub axiom: Token,
    pub ast_rules: Vec<Rule>,
    pub new_non_terminal_reverse: HashMap<Token, BTreeSet<Token>>,
    pub new_non_terminals_subset: Vec<Token>,
    pub reduction_tree: ReductionTree,
    pub foobar: HashMap<Token, ReductionTree>,
    pub old_axiom: Token,
    id_counter: IdCounter,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ReductionNode {
    Node(Token, Vec<ReductionNode>),
    Rule(Rule),
}

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
        let mut token_reverse: BTreeMap<String, (Token, TokenTypes)> = BTreeMap::new();
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
                                    awaiting = Some(TokenTypes::Terminal);
                                } else if buf.eq("nonterminal") {
                                    awaiting = Some(TokenTypes::NonTerminal);
                                } else if buf.eq("axiom") {
                                    awaiting = Some(TokenTypes::Axiom);
                                } else {
                                    return Err(GrammarError::from(format!("Invalid keyword : {}", buf.as_str())));
                                }
                                buf.clear();
                            }
                            SymbolParserState::InIdent => {
                                if let Some(t) = awaiting {
                                    match t {
                                        TokenTypes::Terminal => {
                                            if lexical_sync.contains(&buf) {
                                                let i = lexical_sync.iter().position(|x| x == &buf).unwrap();
                                                token_reverse.insert(buf.clone(), (i, TokenTypes::Terminal));
                                            } else {
                                                token_reverse.insert(buf.clone(), (id_counter.gen_id(), TokenTypes::Terminal));
                                            }
                                        }
                                        TokenTypes::Axiom => axiom = Some(token_reverse.get(buf.as_str()).unwrap().0),
                                        TokenTypes::NonTerminal => {
                                            token_reverse.insert(buf.clone(), (id_counter.gen_id(), TokenTypes::NonTerminal));
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
        let mut token_map: Vec<String> = Vec::new();
        for (raw, (id, token_type)) in &token_reverse {
            token_types.insert(*id, *token_type);
            token_raw.insert(*id, raw.clone());
            token_map.push(raw.clone());
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
            if *v == TokenTypes::NonTerminal {
                non_terminals.push(*id);
            } else if *v == TokenTypes::Terminal {
                terminals.push(*id);
            }
        }

        Ok(RawGrammar {
            rules,
            token_map,
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

impl Rule {
    pub fn new() -> Self {
        Self {
            left: 0,
            right: Vec::new(),
            nesting_rules: Vec::new(),
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
    pub token_map: Vec<String>,
    pub token_reverse: BTreeMap<String, (Token, TokenTypes)>,
    pub ast_rules: Vec<Rule>,
    pub new_non_terminals_subset: Vec<Token>,
    pub new_non_terminal_reverse: HashMap<Token, BTreeSet<Token>>,
    pub reduction_tree: ReductionTree,
    pub new_reduction_tree: ReductionTree,
    pub foobar: HashMap<Token, ReductionTree>,
    pub old_axiom: Token,
    op_table: HashMap<Token, HashMap<Token, Associativity>>,
}

#[allow(unused)]
impl OpGrammar {
    pub fn from(path: &str, lexical_sync: Vec<String>) -> OpGrammar {
        let raw = RawGrammar::from(path, lexical_sync).unwrap();
        OpGrammar::new(raw).unwrap()
    }

    pub fn new(mut g: RawGrammar) -> Result<OpGrammar, GrammarError> {
        let mut inverse_rewrite_rules: HashMap<Token, Vec<Token>> = HashMap::new();
        let mut op_table: HashMap<Token, HashMap<Token, Associativity>> = HashMap::new();

        let delim = g.gen_id();

        g.token_raw.insert(delim, String::from("_DELIM"));
        g.token_reverse.insert(String::from("_DELIM"), (delim, TokenTypes::NonTerminal));

        // Validate that the grammar is in OPG form
        let repeated_rules = g.get_repeated_rhs();
        if let Some(repeated_rules) = repeated_rules {
            return Err(GrammarError::from(
                "Cannot build OP Grammar from grammar with repeated right hand side.".to_string(),
            ));
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
            debug!("{:?} -> {:?}", g.token_raw.get(row).unwrap(), row_full_raw,);
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
            let row_full_raw: Vec<&String> = first_ops.get(row).unwrap().iter().map(|row_item| g.token_raw.get(row_item).unwrap()).collect();
            debug!("{:s_len$} : {:?}", g.token_raw.get(row).unwrap(), row_full_raw, s_len = largest);
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
            let row_full_raw: Vec<&String> = last_ops.get(row).unwrap().iter().map(|row_item| g.token_raw.get(row_item).unwrap()).collect();
            debug!("{:s_len$} : {:?}", g.token_raw.get(row).unwrap(), row_full_raw, s_len = largest);
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
                    if g.terminals.contains(r.right.get(i).unwrap()) && g.terminals.contains(r.right.get(i + 1).unwrap()) {
                        op_table.get_mut(r.right.get(i).unwrap()).unwrap().insert(*r.right.get(i + 1).unwrap(), Equal);
                    }
                    if g.terminals.contains(r.right.get(i).unwrap()) && g.non_terminals.contains(r.right.get(i + 1).unwrap()) {
                        if first_ops.contains_key(r.right.get(i + 1).unwrap()) {
                            let first_op_a = first_ops.get(r.right.get(i + 1).unwrap()).unwrap();
                            for q2 in first_op_a {
                                op_table.get_mut(r.right.get(i).unwrap()).unwrap().insert(*q2, Left);
                            }
                        }
                    }
                    if g.non_terminals.contains(r.right.get(i).unwrap()) && g.terminals.contains(r.right.get(i + 1).unwrap()) {
                        if last_ops.contains_key(r.right.get(i).unwrap()) {
                            let last_op_a = last_ops.get(r.right.get(i).unwrap()).unwrap();
                            for q2 in last_op_a {
                                op_table.get_mut(q2).unwrap().insert(*r.right.get(i + 1).unwrap(), Right);
                            }
                        }
                    }
                    if i + 2 < r.right.len() {
                        if g.terminals.contains(r.right.get(i).unwrap())
                            && g.non_terminals.contains(r.right.get(i + 1).unwrap())
                            && g.terminals.contains(r.right.get(i + 2).unwrap())
                        {
                            op_table.get_mut(r.right.get(i).unwrap()).unwrap().insert(*r.right.get(i + 2).unwrap(), Equal);
                        }
                    }
                }
            }
        }

        op_table.insert(
            delim,
            template
                .clone()
                .into_iter()
                .map(|(t, a)| -> (Token, Associativity) {
                    return (t, Associativity::Right);
                })
                .collect(),
        );
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
            token_map: g.token_map,
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
            new_non_terminal_reverse: g.new_non_terminal_reverse,
            new_non_terminals_subset: g.new_non_terminals_subset,
            reduction_tree: g.reduction_tree,
            new_reduction_tree: tree,
            foobar: g.foobar,
            old_axiom: g.old_axiom,
        })
    }

    pub fn token_list_to_string(value: &Vec<Token>, token_raw: &HashMap<Token, String>) -> Vec<String> {
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
