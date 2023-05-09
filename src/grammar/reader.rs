use crate::grammar::error::GrammarError;
use crate::grammar::reader::TokenTypes::{Axiom, NonTerminal, Terminal};
use crate::grammar::{OpGrammar, Rule, Token};
use log::trace;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::{BufReader, Read};
use std::prelude::rust_2015;

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
    pub fn new() -> Self {
        Self { highest_id: 0 }
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
    id_counter: IdCounter,
}

impl RawGrammar {
    pub fn from(path: &str) -> Result<(Self), Box<dyn Error>> {
        let mut file = fs::File::open(path).unwrap();
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        Ok(RawGrammar::new(buf.as_str())?)
    }
    pub fn new(s: &str) -> Result<RawGrammar, GrammarError> {
        let mut state = GeneralState::ParserSymbols;
        let mut symbol_parser_state = SymbolParserState::InData;
        let mut rule_parser_state = RuleParserState::InData;
        let mut previous: char = 0 as char;
        let mut buf = String::new();
        let mut awaiting: Option<TokenTypes> = None;
        let mut token_reverse: HashMap<String, (Token, TokenTypes)> = HashMap::new();
        let mut axiom: Option<Token> = None;
        let mut id_counter = IdCounter::new();

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
                                            token_reverse.insert(buf.clone(), (id_counter.gen_id(), Terminal));
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
                        return Err(GrammarError::from(format!(
                            "Invalid character in grammar definition: {}",
                            c
                        )))
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
                        }
                        RuleParserState::InRuleRight | RuleParserState::AwaitingRuleRight | RuleParserState::InData => {
                            ()
                        }
                    },
                    ':' | '|' => match rule_parser_state {
                        RuleParserState::InData => {
                            return Err(GrammarError::from("Identifier should precede :.".to_string()))
                        }
                        RuleParserState::InRuleLeft | RuleParserState::AwaitingRuleRight => {
                            rule_parser_state = RuleParserState::InRuleRight;
                            rule.as_mut().unwrap().right.clear();
                        }
                        RuleParserState::InRuleRight => {
                            return Err(GrammarError::from("Illegal char : in right rule".to_string()))
                        }
                        RuleParserState::InRuleIdentifierRight => {
                            return Err(GrammarError::from("Illegal char : in right rule".to_string()))
                        }
                    },
                    '\n' => match rule_parser_state {
                        RuleParserState::InData | RuleParserState::AwaitingRuleRight => (),
                        RuleParserState::InRuleRight => {
                            rule_parser_state = RuleParserState::AwaitingRuleRight;
                            if let Some(r) = rule.clone() {
                                rules.push(r.clone());
                            } else {
                                return Err(GrammarError::from(
                                    "Semicolon used without rule preceding it.".to_string(),
                                ));
                            }
                        }
                        RuleParserState::InRuleIdentifierRight => {
                            rule_parser_state = RuleParserState::AwaitingRuleRight;
                            let (id, _) = token_reverse.get(&*buf).unwrap();
                            rule.as_mut().unwrap().right.push(*id);
                            rules.push(rule.as_mut().unwrap().clone());
                        }
                        RuleParserState::InRuleLeft => {
                            return Err(GrammarError::from("Unexected new line after left rule.".to_string()))
                        }
                    },
                    ';' => {
                        rule_parser_state = RuleParserState::InData;
                        rule = None;
                    }
                    'A'..='Z' | 'a'..='z' | '0'..='9' | '_' => match rule_parser_state {
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
                        RuleParserState::InRuleIdentifierRight => buf.push(c),
                        RuleParserState::AwaitingRuleRight => {
                            return Err(GrammarError::from(
                                "Expected :, | or ;, found start of identifier.".to_string(),
                            ))
                        }
                    },
                    _ => {
                        return Err(GrammarError::from(format!(
                            "Invalid character in grammar definition: {}",
                            c
                        )))
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

        for r in &rules {
            trace!(
                "Rule : {} -> {:?}",
                &token_raw.get(&r.left).unwrap(),
                OpGrammar::token_list_to_string(&r.right, &token_raw)
            );
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
        })
    }
    pub fn gen_id(&mut self) -> Token {
        self.id_counter.gen_id()
    }
}
