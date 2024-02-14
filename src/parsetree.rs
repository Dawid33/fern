use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::fs::File;
use std::{
    borrow::Cow,
    io::{self, Write},
};

use flexi_logger::Logger;

use log::trace;
#[cfg(not(test))]
use log::{info, warn}; // Use log crate when building application

#[cfg(test)]
use std::{println as info, println as warn}; // Workaround to use prinltn! for logs.

type Token = usize;
pub type Id = usize;

#[derive(Debug, Clone)]
pub struct Node {
    pub token: usize,
    pub child_count: usize,
}

impl Node {
    fn new(token: usize) -> Self {
        Self { token, child_count: 0 }
    }
}

/// An append only tree of Tokens. Root is at id 0.
pub struct ParseTree {
    pub nodes: Vec<Node>,
    pub token_map: BTreeMap<Token, String>,
}

impl ParseTree {
    pub fn new(token_map: BTreeMap<Token, String>) -> Self {
        Self { nodes: Vec::new(), token_map }
    }

    pub fn push(&mut self, token: Token) -> Id {
        self.nodes.push(Node::new(token));
        return self.nodes.len() - 1;
    }

    pub fn reduce(&mut self, parent: Token, children: &[Id]) -> Id {
        let m = children.iter().max().unwrap();
        let mut p = Node::new(parent);
        p.child_count = children.len();
        if *m + 1 >= self.nodes.len() {
            self.nodes.push(p);
        } else {
            self.nodes.insert(*m + 1, p);
        }
        *m + 1
    }

    fn pre_order_traverse<F: FnMut(&Vec<(Option<usize>, usize)>, usize)>(&self, mut f: F) {
        let mut stack = Vec::from(&[(None, self.nodes.last().unwrap().child_count)]);

        for (i, n) in self.nodes.iter().enumerate().rev() {
            let last = stack.last_mut().unwrap();
            last.1 -= 1;

            f(&stack, i);

            if n.child_count > 0 {
                stack.push((Some(i), n.child_count as usize));
            } else {
                while let Some((_, last)) = stack.last() {
                    if *last == 0 {
                        stack.pop();
                    } else {
                        break;
                    }
                }
            }
        }
    }

    pub fn dot<W: Write>(&self, out: &mut W) -> io::Result<()> {
        let nodes = self.nodes.iter().map(|n| self.token_map.get(&n.token).unwrap().clone()).collect();
        let mut edges = Vec::new();

        self.pre_order_traverse(|stack, current| {
            if stack.last().unwrap().0.is_some() {
                edges.push((stack.last().unwrap().0.unwrap(), current));
            }
        });
        let g = Graph { nodes, edges };
        dot::render(&g, out)
    }

    pub fn print(&self) {
        self.pre_order_traverse(|stack, current| {
            let n = &self.nodes[current];

            let mut padding = String::new();
            for i in 0..stack.len() - 1 {
                let current = stack.get(i).unwrap();
                if current.1 > 0 {
                    padding.push_str("| ");
                } else {
                    padding.push_str("  ");
                }
            }

            let last = stack.last().unwrap();
            if last.1 > 0 {
                info!("{}├─{}", padding, self.token_map.get(&n.token).unwrap());
            } else {
                info!("{}└─{}", padding, self.token_map.get(&n.token).unwrap());
            }
        });
    }
}

#[test]
fn tree_check_size() {
    println!("Size of Token: {} bytes", std::mem::size_of::<Node>());
    println!("Size of ParseTree: {} bytes", std::mem::size_of::<ParseTree>());
    assert!(true);
}

const MAP: &[&str] = &["A", "B", "C", "D"];

#[test]
fn tree_traverse() {
    let token_map: Vec<String> = MAP.iter().map(|s| s.to_string()).collect();
    let mut tree = ParseTree::new(token_map);
}

#[test]
fn tree_add_child() {}

type Nd = (usize, String);
type Ed = (Nd, Nd);
struct Graph {
    nodes: Vec<String>,
    edges: Vec<(usize, usize)>,
}

impl<'a> dot::Labeller<'a, Nd, Ed> for Graph {
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new("ParseTree").unwrap()
    }
    fn node_id(&'a self, n: &Nd) -> dot::Id<'a> {
        dot::Id::new(format!("N{}", n.0)).unwrap()
    }
    fn node_label(&self, n: &Nd) -> dot::LabelText {
        dot::LabelText::LabelStr(n.1.clone().into())
    }
    fn edge_label(&self, _: &Ed) -> dot::LabelText {
        dot::LabelText::LabelStr("".into())
    }
}

impl<'a> dot::GraphWalk<'a, Nd, Ed> for Graph {
    fn nodes(&'a self) -> dot::Nodes<'a, Nd> {
        let nodes = self.nodes.clone().into_iter().enumerate().collect();
        Cow::Owned(nodes)
    }
    fn edges(&'a self) -> dot::Edges<'a, Ed> {
        self.edges
            .iter()
            .map(|&(i, j)| ((i, self.nodes[i].clone()), (j, self.nodes[j].clone())))
            .collect()
    }
    fn source(&self, e: &Ed) -> Nd {
        e.0.clone()
    }
    fn target(&self, e: &Ed) -> Nd {
        e.1.clone()
    }
}
