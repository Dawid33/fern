use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::io::BufWriter;
use std::io::Write;
use std::process::Termination;

use dot::Edges;
use dot::Kind;
use log::{info, trace};
use regex_syntax::hir::Hir;
use regex_syntax::hir::HirKind;

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
                        'a'..='z'
                        | 'A'..='Z'
                        | '_'
                        | ' '
                        | '0'..='9'
                        | '*'
                        | '?'
                        | '|'
                        | ';'
                        | '\''
                        | '('
                        | ')'
                        | '['
                        | ']'
                        | '-'
                        | ':'
                        | '.'
                        | '+'
                        | '/'
                        | '{'
                        | '}'
                        | '='
                        | '&'
                        | '^'
                        | '%'
                        | '<'
                        | '>'
                        | '!'
                        | '#'
                        | ',' => current_regex.push(c),
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
            info!("{} = {}", k, v);
        }
    }

    pub fn print(&self) {
        for (k, v) in &self.pairs {
            info!("{} = {:?}", k, v);
        }
    }
}

#[derive(Clone, Debug)]
struct Node {
    terminal: Option<String>,
    edges: HashMap<u8, usize>,
}

impl Node {
    pub fn new(terminal: Option<String>, edges: HashMap<u8, usize>) -> Self {
        Self { terminal, edges }
    }
}

pub struct NFA {
    terminals: HashMap<String, usize>,
    nodes: Vec<Node>,
    start_state: usize,
}

// TODO: Implement Thompsons Construction
impl NFA {
    pub fn from(input: LexicalGrammar) -> Self {
        let nodes = Vec::from(&[Node::new(None, HashMap::new())]);
        let mut nfa = Self {
            nodes,
            start_state: 0,
            terminals: HashMap::new(),
        };

        for (token, regex) in input.pairs {
            nfa.add_regex(token, regex);
        }
        return nfa;
    }

    pub fn add_regex(&mut self, terminal: String, regex: Hir) {
        self.nodes.push(Node::new(Some(terminal.clone()), HashMap::new()));
        let mut node_stack: Vec<(usize, usize, Hir)> = Vec::from(&[(self.start_state, self.nodes.len() - 1, regex)]);

        while let Some((mut start_state, finish_state, hir_node)) = node_stack.pop() {
            match hir_node.kind() {
                HirKind::Literal(literal) => {
                    let mut iter = literal.0.iter().peekable();
                    while let Some(c) = iter.next() {
                        if iter.peek().is_none() {
                            let has_next_state = self.nodes.get_mut(start_state).unwrap().edges.contains_key(c);
                            if has_next_state {
                                let next_id = *self.nodes.get(start_state).unwrap().edges.get(c).unwrap();
                                let next = self.nodes.get_mut(next_id).unwrap();
                                next.edges.insert('\0' as u8, finish_state);
                            } else {
                                let start = self.nodes.get_mut(start_state).unwrap();
                                start.edges.insert(*c, finish_state);
                            }

                            break;
                        }

                        let has_next_state = self.nodes.get_mut(start_state).unwrap().edges.contains_key(c);
                        if has_next_state {
                            let next_id = *self.nodes.get_mut(start_state).unwrap().edges.get(c).unwrap();
                            start_state = next_id;
                        } else {
                            self.nodes.push(Node::new(None, HashMap::new()));
                            let next_id = self.nodes.len() - 1;
                            self.nodes.get_mut(start_state).unwrap().edges.insert(*c, next_id);
                            start_state = next_id;
                        }
                    }
                }
                HirKind::Concat(concat) => {
                    let mut iter = concat.iter().peekable();
                    while let Some(hir) = iter.next() {
                        if iter.peek().is_none() {
                            node_stack.push((start_state, finish_state, hir.clone()));
                            break;
                        }

                        self.nodes.push(Node::new(None, HashMap::new()));
                        let mid_state = self.nodes.len() - 1;
                        node_stack.push((start_state, mid_state, hir.clone()));
                        start_state = mid_state;
                    }
                }
                HirKind::Class(class) => {
                    let start = self.nodes.get_mut(start_state).unwrap();
                    start.edges.insert('a' as u8, finish_state);
                }
                HirKind::Repetition(_) => todo!(),
                HirKind::Empty => todo!(),
                HirKind::Look(_) => {}
                HirKind::Capture(_) => todo!(),
                HirKind::Alternation(_) => todo!(),
            }
        }
    }
}

type Nd = (usize, String);
type Ed = (Nd, Nd, String);
struct Graph {
    nodes: Vec<String>,
    edges: VecDeque<(usize, usize, String)>,
}

impl<'a> dot::Labeller<'a, Nd, Ed> for Graph {
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new("example3").unwrap()
    }
    fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
        dot::Id::new(format!("N{}", n.0)).unwrap()
    }
    fn node_label(&self, n: &Nd) -> dot::LabelText {
        let &(i, _) = n;
        dot::LabelText::HtmlStr(self.nodes[i].clone().into())
    }
    fn edge_label(&self, e: &Ed) -> dot::LabelText {
        let &(_, _, ref lbl) = e;
        dot::LabelText::LabelStr(lbl.clone().into())
    }
}

impl<'a> dot::GraphWalk<'a, Nd, Ed> for Graph {
    fn nodes(&'a self) -> dot::Nodes<'a, Nd> {
        let new_nodes = self.nodes.clone().into_iter().enumerate().collect();
        Cow::Owned(new_nodes)
    }
    fn edges(&'a self) -> dot::Edges<'a, Ed> {
        self.edges
            .iter()
            .map(|&(i, j, ref lbl)| ((i, self.nodes[i].clone()), (j, self.nodes[j].clone()), lbl.clone()))
            .collect()
    }
    fn source(&self, e: &Ed) -> Nd {
        e.0.clone()
    }
    fn target(&self, e: &Ed) -> Nd {
        e.1.clone()
    }
}

pub fn render<W: Write>(nfa: NFA, output: &mut W) {
    let mut nodes: Vec<String> = Vec::new();
    let mut edges = VecDeque::new();
    for (i, n) in nfa.nodes.iter().enumerate() {
        if let Some(terminal) = &n.terminal {
            nodes.push(terminal.clone());
        } else {
            nodes.push(i.to_string());
        }
        info!("{:?}", n.edges);
        for (k, v) in n.edges.iter() {
            edges.push_front((i, *v, format!(" {}", *k as char)));
        }
    }

    let graph = Graph { nodes, edges };
    let mut result = BufWriter::new(Vec::new());
    dot::render(&graph, &mut result).unwrap();
    let bytes = result.into_inner().unwrap();
    let mut string = String::from_utf8(bytes).unwrap();
    let idx = string.find('{').unwrap();
    string.insert_str(idx + 1, "layout=\"dot\"");
    write!(output, "{}", string).unwrap();
}
// TODO: NFA -> DFA using powerset construction https://en.wikipedia.org/wiki/Powerset_construction
