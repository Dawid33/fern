use std::fmt::Debug;
use std::fs::File;
use std::{
    borrow::Cow,
    io::{self, Write},
};

use flexi_logger::Logger;

#[cfg(not(test))]
use log::{info, warn}; // Use log crate when building application

#[cfg(test)]
use std::{println as info, println as warn}; // Workaround to use prinltn! for logs.

#[derive(Debug, Clone)]
pub struct Node {
    token: usize,
    child_count: u16,
}

impl Node {
    fn new(token: usize) -> Self {
        Self { token, child_count: 0 }
    }
}

/// An append only tree of Tokens. Root is at id 0.
pub struct ParseTree {
    child_count: usize,
    nodes: Vec<Node>,
    token_map: Vec<String>,
}

impl ParseTree {
    pub fn new(token_map: Vec<String>) -> Self {
        Self {
            nodes: Vec::new(),
            child_count: 0,
            token_map,
        }
    }

    pub fn with_capacity(size: usize, token_map: Vec<String>) -> Self {
        Self {
            nodes: Vec::with_capacity(size),
            child_count: 0,
            token_map,
        }
    }

    pub fn add_root(&mut self, token: usize) -> usize {
        self.nodes.push(Node::new(token));
        self.child_count += 1;
        return self.nodes.len() - 1;
    }

    /// Append a child to token at position id
    pub fn add_child(&mut self, id: usize, token: usize) -> usize {
        let parent = match self.nodes.get_mut(id) {
            Some(id) => id,
            None => panic!("Failed to append child to parse tree: Requested parent doesn't exist."),
        };
        let child_index = id + parent.child_count as usize + 1;
        parent.child_count += 1;
        self.nodes.insert(child_index, Node::new(token));
        return child_index;
    }

    pub fn traverse<F: FnMut(&Vec<(Option<usize>, usize)>, usize)>(&self, mut f: F) {
        let mut stack = Vec::from(&[(None, self.child_count)]);

        for (i, n) in self.nodes.iter().enumerate() {
            let last = stack.last_mut().unwrap();
            last.1 -= 1;

            f(&stack, i);

            let (_, last) = stack.last_mut().unwrap();
            if n.child_count > 0 {
                stack.push((Some(i), n.child_count as usize));
            } else {
                if *last == 0 {
                    stack.pop();
                }
            }
        }
    }

    pub fn dot<W: Write>(&self, out: &mut W) -> io::Result<()> {
        let nodes = self.nodes.iter().map(|n| self.token_map[n.token].clone()).collect();
        let mut edges = Vec::new();

        self.traverse(|stack, current| {
            if stack.last().unwrap().0.is_some() {
                edges.push((stack.last().unwrap().0.unwrap(), current));
            }
        });
        println!("{:?}", edges);
        let g = Graph { nodes, edges };
        dot::render(&g, out)
    }

    pub fn print(&self) {
        self.traverse(|stack, current| {
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
                info!("{}├─{}", padding, self.token_map[n.token]);
            } else {
                info!("{}└─{}", padding, self.token_map[n.token]);
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
    let a = tree.add_root(0);
    let b = tree.add_child(a, 1);
    let d = tree.add_child(b, 3);
    let c = tree.add_child(a, 2);
    let c = tree.add_child(a, 2);
    let a = tree.add_root(1);
}

#[test]
fn tree_add_child() {
    let token_map: Vec<String> = MAP.iter().map(|s| s.to_string()).collect();
    let mut tree = ParseTree::new(token_map);
    let a = tree.add_root(0);
    let b = tree.add_child(a, 1);
    let d = tree.add_child(b, 3);
    let c = tree.add_child(a, 2);
    let c = tree.add_child(a, 2);
    let a = tree.add_root(1);
    tree.print();
    let mut f = File::create("ptree.dot").unwrap();
    tree.dot(&mut f).unwrap();
    assert!(false);
}

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
