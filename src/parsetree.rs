use std::collections::HashMap;
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
    token: usize,
    child_count: usize,
    total: usize,
}

impl Node {
    fn new(token: usize, total: usize) -> Self {
        Self { token, child_count: 0, total }
    }
}

/// An append only tree of Tokens. Root is at id 0.
pub struct ParseTree {
    child_count: usize,
    insertion_point: usize,
    nodes: Vec<Node>,
    stack: Vec<usize>,
    token_map: HashMap<Token, String>,
}

impl ParseTree {
    pub fn new(token_map: HashMap<Token, String>) -> Self {
        Self {
            nodes: Vec::new(),
            child_count: 0,
            insertion_point: 0,
            stack: Vec::new(),
            token_map,
        }
    }

    /// Take a list of nodes and create a parent over them. Returns the id of the parent.
    pub fn reduce(&mut self, parent: Token, children: &[(Option<Id>, Token)]) -> Id {
        let mut parent_total = 0;
        let mut pivot: Option<Id> = None;
        let mut b = String::new();
        for (_, t) in children {
            b.push_str(format!("{} ", self.token_map.get(t).unwrap()).as_str());
        }
        warn!("children: {}", b);

        for (existing_id, new_token) in children.iter().rev() {
            if let Some(id) = existing_id {
                let total = self.nodes.get(*id).unwrap().total;
                parent_total += total;
                pivot = Some((id + 1) - total);

                warn!(
                    "pivot: ({}, token: {}, total: {}, children: {})",
                    (id + 1) - total,
                    self.token_map.get(&self.nodes.get((id + 1) - total).unwrap().token).unwrap(),
                    &self.nodes.get(*id).unwrap().total,
                    &self.nodes.get(*id).unwrap().child_count
                );
            } else if let Some(id) = pivot {
                self.nodes.insert(id, Node::new(*new_token, 1));
                parent_total += 1;
            } else {
                self.nodes.push(Node::new(*new_token, 1));
                parent_total += 1;
            }
        }

        let m = children.iter().min().unwrap();
        let mut p = Node::new(parent, parent_total + 1);
        p.child_count = children.len();
        self.nodes.push(p);

        let mut b = String::new();
        for n in &self.nodes {
            b.push_str(format!("({}, {}) ", self.token_map.get(&n.token).unwrap(), n.total).as_str());
        }
        warn!("after: {}", b);

        self.nodes.len() - 1
    }

    pub fn pre_order_traverse<F: FnMut(&Vec<(Option<usize>, usize)>, usize)>(&self, mut f: F) {
        let mut stack = Vec::from(&[(None, self.nodes.last().unwrap().child_count)]);

        for (i, n) in self.nodes.iter().enumerate().rev() {
            warn!("{:?}", stack);
            let n_t = &self.nodes[i];
            let last = stack.last_mut().unwrap();
            // warn!(
            //     "last.1: {}, token: {}, children: {}",
            //     last.1,
            //     self.token_map.get(&n_t.token).unwrap(),
            //     n_t.child_count
            // );
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
        println!("{:?}", edges);
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
    // let a = tree.add_root(0);
    // let b = tree.add_child(a, 1);
    // let d = tree.add_child(b, 3);
    // let c = tree.add_child(a, 2);
    // let c = tree.add_child(a, 2);
    // let a = tree.add_root(1);
}

#[test]
fn tree_add_child() {
    let token_map: Vec<String> = MAP.iter().map(|s| s.to_string()).collect();
    let mut tree = ParseTree::new(token_map);
    let a1 = tree.add_node(1);
    let a2 = tree.add_node(1);
    let a3 = tree.add_node(1);
    let p1 = tree.reduce(0, &[a1, a2, a3]);

    let a1 = tree.add_node(1);
    let a2 = tree.add_node(1);
    let a3 = tree.add_node(1);
    let p2 = tree.reduce(0, &[a1, a2, a3]);
    let _ = tree.reduce(0, &[p1, p2]);
    tree.print();
    // let mut f = File::create("ptree.dot").unwrap();
    // tree.dot(&mut f).unwrap();
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
