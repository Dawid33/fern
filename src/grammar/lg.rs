use std::borrow::Cow;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::io::BufWriter;
use std::io::Write;
use std::process::Termination;

use dot::Edges;
use dot::Kind;
use log::warn;
use log::{info, trace};
use regex_syntax::ast::Alternation;
use regex_syntax::hir::Class;
use regex_syntax::hir::Hir;
use regex_syntax::hir::HirKind;

pub type State = usize;
pub type Token = usize;

enum ParserState {
    InWord,
    InRegex,
    InRegexEscape,
    AwaitingEquals,
    AwaitingWord,
    AwaitingRegex,
}

#[derive(Clone)]
pub struct LexicalGrammar {
    pairs: BTreeMap<String, Hir>,
}

impl LexicalGrammar {
    pub fn from(input: String) -> Self {
        let token_regex_pairs = Self::scanner(input);

        let mut pairs: BTreeMap<String, Hir> = BTreeMap::new();
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
        let mut state = ParserState::AwaitingWord;

        for c in input.chars() {
            let mut reconsume = true;
            while reconsume {
                reconsume = false;
                match state {
                    ParserState::InWord => match c {
                        'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => current_token.push(c),
                        ' ' => state = ParserState::AwaitingEquals,
                        _ => panic!("Illegal character in lexical grammar token identifier. Found {}", c),
                    },
                    ParserState::AwaitingEquals => match c {
                        ' ' => {}
                        '\"' => state = ParserState::InRegex,
                        '=' => state = ParserState::AwaitingRegex,
                        _ => panic!("Only whitespace is allowed between identifier and equals sign. Found {}", c),
                    },
                    ParserState::AwaitingRegex => match c {
                        ' ' => {}
                        '\"' => state = ParserState::InRegex,
                        _ => panic!("Only whitespace is allowed between equals sign regex. Found {}", c),
                    },
                    ParserState::InRegex => match c {
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
                            state = ParserState::AwaitingWord;
                            if let Some(value) = pairs.insert(current_token, current_regex) {
                                panic!("Token already defined. Found {}", value);
                            }
                            current_token = String::new();
                            current_regex = String::new();
                        }
                        '\\' => state = ParserState::InRegexEscape,
                        _ => panic!("Illegal character in regular expression. Found '{}'", c),
                    },
                    ParserState::InRegexEscape => match c {
                        '\"' | '\\' => {
                            // Regex parser has its own thing for escape sequences.
                            // This scanner only cares about excaping single quotes.
                            if c == '\\' {
                                current_regex.push(c);
                            }
                            current_regex.push(c);
                            state = ParserState::InRegex;
                        }
                        _ => {
                            current_regex.push('\\');
                            reconsume = true;
                            state = ParserState::InRegex;
                        }
                    },
                    ParserState::AwaitingWord => match c {
                        'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                            reconsume = true;
                            state = ParserState::InWord;
                        }
                        ' ' | '\n' => {}
                        _ => panic!("Only whitespace and newline is allowed between regex and next token. Found {}", c),
                    },
                }
            }
        }
        return pairs;
    }

    pub fn get_tokens(&self) -> Vec<String> {
        self.pairs.clone().into_keys().collect()
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
    eplisons: Vec<usize>,
}

impl Node {
    pub fn new(terminal: Option<String>, edges: HashMap<u8, usize>) -> Self {
        Self {
            terminal,
            edges,
            eplisons: Vec::new(),
        }
    }
}

pub struct StateGraph {
    terminals: HashMap<String, usize>,
    nodes: Vec<Node>,
    start_state: usize,
    grammar: LexicalGrammar,
    start_states: Vec<State>,
}

// TODO: Implement Thompsons Construction
impl StateGraph {
    pub fn from(grammar: LexicalGrammar) -> Self {
        let nodes = Vec::from(&[Node::new(None, HashMap::new())]);
        let mut nfa = Self {
            nodes,
            start_state: 0,
            terminals: HashMap::new(),
            grammar,
            start_states: Vec::new(),
        };

        for (token, regex) in nfa.grammar.pairs.clone() {
            nfa.add_regex(token, regex);
        }
        nfa.find_start_states();
        return nfa;
    }

    pub fn add_regex(&mut self, terminal: String, regex: Hir) {
        self.nodes.push(Node::new(Some(terminal.clone()), HashMap::new()));
        info!("{:?}", &regex);
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
                                next.eplisons.push(finish_state);
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
                    if let Class::Unicode(ranges) = class {
                        let mut chars = Vec::new();
                        for range in ranges.iter() {
                            for c in range.start()..=range.end() {
                                chars.push(c);
                            }
                        }
                        let start = self.nodes.get_mut(start_state).unwrap();
                        for c in chars {
                            start.edges.insert(c as u8, finish_state);
                        }
                    } else {
                        panic!("Lexer doesn't support unicode ranges in regex's.");
                    }
                }
                HirKind::Repetition(rep) => {
                    // info!("{:?}", rep);
                    self.nodes.push(Node::new(None, HashMap::new()));
                    let inner_start_id = self.nodes.len() - 1;
                    self.nodes.push(Node::new(None, HashMap::new()));
                    let inner_finish_id = self.nodes.len() - 1;

                    let start = self.nodes.get_mut(start_state).unwrap();
                    start.eplisons.push(finish_state);
                    start.eplisons.push(inner_start_id);

                    let inner_finish = self.nodes.get_mut(inner_finish_id).unwrap();
                    inner_finish.eplisons.push(inner_start_id);
                    inner_finish.eplisons.push(finish_state);

                    node_stack.push((inner_start_id, inner_finish_id, *rep.sub.clone()));
                }
                HirKind::Capture(capture) => {
                    node_stack.push((start_state, finish_state, *capture.sub.clone()));
                }
                HirKind::Alternation(alternation) => {
                    for hir in alternation {
                        self.nodes.push(Node::new(None, HashMap::new()));
                        let inner_start_id = self.nodes.len() - 1;
                        self.nodes.push(Node::new(None, HashMap::new()));
                        let inner_finish_id = self.nodes.len() - 1;

                        let start = self.nodes.get_mut(start_state).unwrap();
                        start.eplisons.push(inner_start_id);

                        let inner_finish = self.nodes.get_mut(inner_finish_id).unwrap();
                        inner_finish.eplisons.push(finish_state);

                        node_stack.push((inner_start_id, inner_finish_id, hir.clone()));
                    }
                }
                HirKind::Empty => todo!(),
                HirKind::Look(_) => todo!(),
            }
        }
    }

    // POWERRRRR SSSSEEEEEEEETTTT CONSTRUCTIONNNNN!!1!1!1
    // Traverse the graph and follow eplison rules to find sets of states
    pub fn convert_to_dfa(self) -> StateGraph {
        let (c, edges, terminal) = self.get_transitive_closure(BTreeSet::from([0]));
        // Populate dfa graph with one node for the root at index 0.
        let mut dfa: Vec<Node> = Vec::from(&[Node {
            terminal,
            edges: HashMap::new(),
            eplisons: Vec::new(),
        }]);
        let mut node_map: HashMap<BTreeSet<usize>, usize> = HashMap::from([(c, 0)]);
        let mut stack: Vec<(usize, (u8, BTreeSet<usize>))> = Vec::new();
        for e in edges {
            stack.push((0, e));
        }

        let mut cnt = 0;
        while let Some((previous, (letter, next_nodes))) = stack.pop() {
            let (states_closure, next_edges, terminal) = self.get_transitive_closure(next_nodes);

            let index = if node_map.contains_key(&states_closure) {
                *node_map.get(&states_closure).unwrap()
            } else {
                dfa.push(Node {
                    terminal,
                    edges: HashMap::new(),
                    eplisons: Vec::new(),
                });
                let index = dfa.len() - 1;
                node_map.insert(states_closure, dfa.len() - 1);
                index
            };

            let prev = dfa.get_mut(previous).unwrap();
            let result = prev.edges.insert(letter, index);

            if let None = result {
                for e in next_edges {
                    stack.push((index, e));
                }
            }

            // info!("dfa {:?}", dfa);
            if cnt > 10 {
                // break;
            }
            cnt += 1;
            // info!("stack {:?}", stack);
        }
        StateGraph {
            terminals: self.terminals,
            nodes: dfa,
            start_state: 0,
            grammar: self.grammar,
        }
    }

    fn get_transitive_closure(&self, mut states: BTreeSet<usize>) -> (BTreeSet<usize>, HashMap<u8, BTreeSet<usize>>, Option<String>) {
        let mut confirmed_states: BTreeSet<usize> = BTreeSet::from(states.clone());
        let mut confirmed_edges: HashMap<u8, BTreeSet<usize>> = HashMap::new();
        let mut terminal = None;
        let mut cnt = 0;
        while !states.is_empty() {
            let mut iter = states.into_iter();
            let mut to_push: Vec<usize> = Vec::new();
            while let Some(id) = iter.next() {
                let node = self.nodes.get(id).unwrap();
                if node.terminal.is_some() {
                    if let Some(t) = terminal.clone() {
                        if t != *node.terminal.as_ref().unwrap() {
                            panic!("Cannot have dfa node with more than one terminal.");
                        }
                    } else {
                        // info!("terminal: {}", id);
                        terminal = node.terminal.clone();
                    }
                }
                for next_state in &node.eplisons {
                    to_push.push(*next_state);
                }
                for (letter, other) in &node.edges {
                    if confirmed_edges.contains_key(&letter) {
                        let set = confirmed_edges.get_mut(&letter).unwrap();
                        set.insert(*other);
                    } else {
                        confirmed_edges.insert(*letter, BTreeSet::from([*other]));
                    }
                }
            }
            states = BTreeSet::new();
            for node in to_push {
                states.insert(node);
                confirmed_states.insert(node);
            }
            cnt += 1;
            if cnt > 5 {
                // break;
            }
        }
        trace!("states: {:?}", confirmed_states);
        // info!("edges: {:?}", confirmed_edges);
        return (confirmed_states, confirmed_edges, terminal);
    }

    pub fn build_table(&self) -> LexingTable {
        let mut map = HashMap::new();
        let terminal_map = self.grammar.get_tokens();
        for (i, t) in terminal_map.iter().enumerate() {
            map.insert(t, i);
        }
        let mut terminals = HashMap::new();
        let mut table: HashMap<u8, HashMap<usize, usize>> = HashMap::new();
        for (i, n) in self.nodes.iter().enumerate() {
            if !n.eplisons.is_empty() {
                panic!("cannot build table with graph that has epsilon transitions. Make sure its a dfa");
            }
            if let Some(t) = &n.terminal {
                terminals.insert(i, *map.get(&t).unwrap());
            }
            for (letter, state) in &n.edges {
                if table.contains_key(letter) {
                    table.get_mut(letter).unwrap().insert(i, *state);
                } else {
                    table.insert(*letter, HashMap::from([(i, *state)]));
                }
            }
        }
        LexingTable {
            table,
            terminals,
            terminal_map,
            sub_tables: HashMap::new(),
        }
    }

    pub fn find_start_states(&mut self) {}
}

#[derive(Debug, Clone)]
pub struct LexingTable {
    table: HashMap<u8, HashMap<usize, usize>>,
    terminals: HashMap<State, Token>,
    pub sub_tables: HashMap<Token, (LexingTable, usize)>,
    pub terminal_map: Vec<String>,
}

pub enum LookupResult {
    Terminal(usize),
    State(usize),
    Err,
}

impl LexingTable {
    pub fn add_table(&mut self, on_word: Token, table: LexingTable) {
        let offset = self.terminal_map.len();
        self.terminal_map.extend(table.terminal_map.clone());
        self.sub_tables.insert(on_word, (table, offset));
    }
    pub fn try_get_terminal(&self, state: usize) -> Option<usize> {
        if let Some(t) = self.terminals.get(&state) {
            Some(*t)
        } else {
            None
        }
    }
    pub fn get(&self, input: u8, state: usize) -> LookupResult {
        let letter = if let Some(map) = self.table.get(&input) {
            map
        } else {
            return LookupResult::Err;
        };

        if letter.contains_key(&state) {
            LookupResult::State(*letter.get(&state).unwrap())
        } else if self.terminals.contains_key(&state) {
            LookupResult::Terminal(*self.terminals.get(&state).unwrap())
        } else {
            LookupResult::Err
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

pub fn render<W: Write>(nfa: &StateGraph, output: &mut W) {
    let mut nodes: Vec<String> = Vec::new();
    let mut edges = VecDeque::new();
    for (i, n) in nfa.nodes.iter().enumerate() {
        if let Some(terminal) = &n.terminal {
            nodes.push(terminal.clone());
        } else {
            nodes.push(i.to_string());
        }
        // info!("{:?}", n.edges);
        for (k, v) in n.edges.iter() {
            edges.push_front((i, *v, format!(" {}", *k as char)));
        }
        for other in n.eplisons.iter() {
            edges.push_front((i, *other, "\\0".to_owned()));
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
