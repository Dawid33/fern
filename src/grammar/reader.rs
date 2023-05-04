use crate::grammar::error::GrammarError;
use crate::grammar::reader::TokenTypes::{Axiom, NonTerminal, Terminal};
use crate::grammar::{Grammar, Rule, Token};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
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

/// Ad-hoc hand written parser for loading in .g grammar files.
pub fn read_grammar_file(s: &str) -> Result<Grammar, GrammarError> {
    let mut state = GeneralState::ParserSymbols;
    let mut symbol_parser_state = SymbolParserState::InData;
    let mut rule_parser_state = RuleParserState::InData;
    let mut previous: char = 0 as char;
    let mut buf = String::new();
    let mut awaiting: Option<TokenTypes> = None;
    let mut tokens: HashMap<String, (Token, TokenTypes)> = HashMap::new();
    let mut axiom: Option<Token> = None;

    let mut highest_id: Token = 0;

    let mut gen_id = || -> Token {
        highest_id += 1;
        return highest_id;
    };

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
                                return Err(GrammarError::from(format!(
                                    "Invalid keyword : {}",
                                    buf.as_str()
                                )));
                            }
                            buf.clear();
                        }
                        SymbolParserState::InIdent => {
                            if let Some(t) = awaiting {
                                match t {
                                    Terminal => {
                                        tokens.insert(buf.clone(), (gen_id(), Terminal));
                                    }
                                    Axiom => axiom = Some(tokens.get(buf.as_str()).unwrap().0),
                                    NonTerminal => {
                                        tokens.insert(buf.clone(), (gen_id(), NonTerminal));
                                    }
                                    _ => {}
                                }
                                awaiting = None;
                                buf.clear();
                            } else {
                                return Err(GrammarError::from(format!(
                                    "Rogue identifier : {}",
                                    buf.as_str()
                                )));
                            }
                        }
                        SymbolParserState::InData => (),
                    }
                    symbol_parser_state = SymbolParserState::InData;
                }
                'A'..='Z' | 'a'..='z' | '0'..='9' => {
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
                        let (id, _) = tokens.get(&*buf).unwrap();
                        rule = Some(Rule::from(*id));
                        rule_parser_state = RuleParserState::AwaitingRuleRight;
                    }
                    RuleParserState::InRuleIdentifierRight => {
                        let (id, _) = tokens.get(&*buf).unwrap();
                        rule.as_mut().unwrap().right.push(*id);
                        rule_parser_state = RuleParserState::InRuleRight;
                    }
                    RuleParserState::InRuleRight
                    | RuleParserState::AwaitingRuleRight
                    | RuleParserState::InData => (),
                },
                ':' | '|' => match rule_parser_state {
                    RuleParserState::InData => {
                        return Err(GrammarError::from(
                            "Identifier should precede :.".to_string(),
                        ))
                    }
                    RuleParserState::InRuleLeft | RuleParserState::AwaitingRuleRight => {
                        rule_parser_state = RuleParserState::InRuleRight;
                        rule.as_mut().unwrap().right.clear();
                    }
                    RuleParserState::InRuleRight => {
                        return Err(GrammarError::from(
                            "Illegal char : in right rule".to_string(),
                        ))
                    }
                    RuleParserState::InRuleIdentifierRight => {
                        return Err(GrammarError::from(
                            "Illegal char : in right rule".to_string(),
                        ))
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
                        let (id, _) = tokens.get(&*buf).unwrap();
                        rule.as_mut().unwrap().right.push(*id);
                        rules.push(rule.as_mut().unwrap().clone());
                    }
                    RuleParserState::InRuleLeft => {
                        return Err(GrammarError::from(
                            "Unexected new line after left rule.".to_string(),
                        ))
                    }
                },
                ';' => {
                    rule_parser_state = RuleParserState::InData;
                    rule = None;
                }
                'A'..='Z' | 'a'..='z' | '0'..='9' => match rule_parser_state {
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

    let mut t_type: HashMap<Token, TokenTypes> = HashMap::new();
    let mut t_raw: HashMap<Token, String> = HashMap::new();
    for (raw, (id, token_type)) in &tokens {
        t_type.insert(*id, *token_type);
        t_raw.insert(*id, raw.clone());
    }
    let axiom: Token = axiom.expect("Need to specify and axiom.");

    // for r in &rules {
    //     info!("Rule : {} -> {:?}", &t_raw.get(&r.left).unwrap(), Grammar::token_list_to_string(&r.right, &t_raw));
    // }

    Grammar::new(rules, t_type, t_raw, tokens, axiom, gen_id())
}
