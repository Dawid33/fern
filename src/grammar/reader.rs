use std::fs;
use std::io::{BufReader, Read};
use crate::grammar::error::GrammarError;
use crate::grammar::Grammar;
use crate::grammar::reader::TokenTypes::{Axiom, NonTerminal, Terminal};

#[derive(Clone, Debug)]
pub struct Rule {
    pub left: u32,
    pub right: Vec<u32>,
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

#[derive(Clone, Debug, Copy)]
enum TokenTypes {
    Terminal,
    Axiom,
    NonTerminal,
}

/// Ad-hoc hand written parser for loading in .g grammar files.
pub fn read_grammar_file(s: &str) -> Result<(), GrammarError> {
    let mut state = GeneralState::ParserSymbols;
    let mut symbol_parser_state = SymbolParserState::InData;
    let mut previous: char = 0 as char;
    let mut buf = String::new();
    let mut awaiting: Option<TokenTypes> = None;
    let mut tokens: Vec<(TokenTypes, String)> = Vec::new();

    for c in s.chars() {
        match state {
            GeneralState::ParserSymbols=> {
                match c {
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
                                    awaiting = Some(Terminal);
                                } else {
                                    return Err(GrammarError::from(format!("Invalid keyword : {}", buf.as_str())));
                                }
                                buf.clear();
                            },
                            SymbolParserState::InIdent => {
                                if let Some(t) = awaiting {
                                    match t {
                                        Terminal => tokens.push((Terminal, buf.clone())),
                                        Axiom => tokens.push((Axiom, buf.clone())),
                                        NonTerminal => tokens.push((NonTerminal, buf.clone())),
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
                    'A' ..= 'Z' | 'a' ..= 'z' => {
                        if symbol_parser_state == SymbolParserState::InData {
                            symbol_parser_state = SymbolParserState::InIdent;
                        }
                        buf.push(c);
                    },
                    _ => {
                        return Err(GrammarError::from(format!("Invalid character in grammar definition: {}", c)))
                    }
                }
            },
            GeneralState::Rules => {

            }
        }
        previous = c;
    }

    Ok(())
}

#[test]
pub fn test_read_grammar_file() {
    let mut file = fs::File::open("json.g").unwrap();
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();

    read_grammar_file(buf.as_str()).unwrap();
}
/*

 */