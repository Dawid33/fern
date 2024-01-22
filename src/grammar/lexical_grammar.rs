use std::collections::HashMap;

use log::trace;
use regex_syntax::hir::Hir;

pub struct LexicalGrammar {
    pairs: HashMap<String, Hir>,
}

enum State {
    InWord,
    InRegex,
    InRegexEscape,
    AwaitingEquals,
    AwaitingWord,
    AwaitingRegex,
}

impl LexicalGrammar {
    pub fn from(input: String) -> Self {
        let token_regex_pairs = Self::scanner(input);
        // Self::print_pairs(&token_regex_pairs);

        let mut pairs: HashMap<String, Hir> = HashMap::new();
        for (token, regex) in token_regex_pairs {
            match regex_syntax::parse(&regex) {
                Ok(r) => pairs.insert(token, r),
                Err(e) => panic!("Failed to parse regular expression : {}. Error : {}", regex, e),
            };
        }
        Self { pairs }
    }

    fn scanner(input: String) -> HashMap<String, String> {
        let mut pairs: HashMap<String, String> = HashMap::new();
        let mut current_token = String::new();
        let mut current_regex = String::new();
        let mut state = State::AwaitingWord;

        for c in input.chars() {
            let mut reconsume = true;
            while reconsume {
                reconsume = false;
                match state {
                    State::InWord => match c {
                        'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => current_token.push(c),
                        ' ' => state = State::AwaitingEquals,
                        _ => panic!("Illegal character in lexical grammar token identifier. Found {}", c),
                    },
                    State::AwaitingEquals => match c {
                        ' ' => {}
                        '\"' => state = State::InRegex,
                        '=' => state = State::AwaitingRegex,
                        _ => panic!("Only whitespace is allowed between identifier and equals sign. Found {}", c),
                    },
                    State::AwaitingRegex => match c {
                        ' ' => {}
                        '\"' => state = State::InRegex,
                        _ => panic!("Only whitespace is allowed between equals sign regex. Found {}", c),
                    },
                    State::InRegex => match c {
                        'a'..='z' | 'A'..='Z' | '_' | ' ' | '0'..='9' | '*' | '?' | '|' | ';' | '\'' => current_regex.push(c),
                        '\"' => {
                            state = State::AwaitingWord;
                            if let Some(value) = pairs.insert(current_token, current_regex) {
                                panic!("Token already defined. Found {}", value);
                            }
                            current_token = String::new();
                            current_regex = String::new();
                        }
                        '\\' => state = State::InRegexEscape,
                        _ => panic!("Illegal character in regular expression. Found '{}'", c),
                    },
                    State::InRegexEscape => match c {
                        '\"' | '\\' => {
                            // Regex parser has its own thing for escape sequences.
                            // This scanner only cares about excaping single quotes.
                            if c == '\\' {
                                current_regex.push(c);
                            }
                            current_regex.push(c);
                            state = State::InRegex;
                        }
                        _ => {
                            current_regex.push('\\');
                            reconsume = true;
                            state = State::InRegex;
                        }
                    },
                    State::AwaitingWord => match c {
                        'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                            reconsume = true;
                            state = State::InWord;
                        }
                        ' ' | '\n' => {}
                        _ => panic!("Only whitespace and newline is allowed between regex and next token. Found {}", c),
                    },
                }
            }
        }
        return pairs;
    }

    pub fn print_pairs(pairs: &HashMap<String, String>) {
        for (k, v) in pairs {
            trace!("{} = {}", k, v);
        }
    }

    pub fn print(&self) {
        for (k, v) in &self.pairs {
            trace!("{} = {:?}", k, v);
        }
    }
}

pub struct NFA {}

impl NFA {
    // TODO: Implement Thompsons Construction https://en.wikipedia.org/wiki/Thompson%27s_construction
    pub fn from(input: LexicalGrammar) -> Self {
        Self {}
    }
}

// TODO: NFA -> DFA using powerset construction https://en.wikipedia.org/wiki/Powerset_construction
