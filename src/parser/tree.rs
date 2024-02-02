use std::fmt::Debug;
use std::{
    borrow::Cow,
    io::{self, Write},
};

use log::info;

#[derive(Copy, Clone, Debug)]
pub struct Node<T>
where
    T: Copy + Clone + Debug,
{
    token: T,
    child_count: u16,
    children_offset: u32,
}

impl<T> Default for Node<T>
where
    T: Default + Copy + Clone + Debug,
{
    fn default() -> Self {
        Self {
            token: T::default(),
            child_count: 0,
            children_offset: 0,
        }
    }
}

impl<T> Node<T>
where
    T: Copy + Clone + Debug,
{
    fn new(token: T) -> Self {
        Self {
            token,
            child_count: 0,
            children_offset: 0,
        }
    }
}

/// An append only tree of Tokens. Root is at id 0.
pub struct ParseTree<T>
where
    T: Copy + Clone + Debug,
{
    tree: Vec<Node<T>>,
}

impl<T> ParseTree<T>
where
    T: Copy + Debug,
{
    pub fn new(root: T) -> Self {
        Self {
            tree: Vec::from(&[Node::new(root)]),
        }
    }

    pub fn with_capacity(root: T, size: usize) -> Self {
        let mut tree = Vec::with_capacity(size);
        tree.push(Node::new(root));
        Self { tree }
    }

    /// Append a child to token at position id
    pub fn add_child(&mut self, id: usize, token: T) -> usize {
        let parent = match self.tree.get_mut(id) {
            Some(id) => id,
            None => panic!("Failed to append child to parse tree: Requested parent doesn't exist."),
        };
        let child_index = (parent.children_offset + parent.child_count as u32 + 1) as usize;
        self.tree.insert(child_index, Node::new(token));
        return child_index;
    }

    pub fn dot<W: Write>(&self, out: &mut W) -> io::Result<()> {
        let nodes = self.tree.iter().map(|n| format!("{:?}", n)).collect();
        let g = Graph { nodes, edges: Vec::new() };
        dot::render(&g, out)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum TestTokenType {
    A,
    B,
    C,
    D,
}

#[test]
fn tree_check_size() {
    println!("Size of Token: {} bytes", std::mem::size_of::<Node<TestTokenType>>());
    println!("Size of ParseTree: {} bytes", std::mem::size_of::<ParseTree<TestTokenType>>());
    assert!(true);
}

#[test]
fn tree_add_child() {
    let mut tree = ParseTree::new(TestTokenType::A);
    tree.add_child(0, TestTokenType::B);
    tree.add_child(1, TestTokenType::C);
    tree.add_child(2, TestTokenType::D);
    let mut f = std::fs::File::create("tree.dot").unwrap();
    tree.dot(&mut f).unwrap();
    assert_eq!(TestTokenType::A, tree.tree.get(0).unwrap().token);
    assert_eq!(TestTokenType::B, tree.tree.get(1).unwrap().token);
    assert_eq!(TestTokenType::C, tree.tree.get(2).unwrap().token);
    assert_eq!(TestTokenType::D, tree.tree.get(3).unwrap().token);
}

type DotNode = (usize, String);
type DotEdge = (DotNode, DotNode);
struct Graph {
    nodes: Vec<String>,
    edges: Vec<(usize, usize)>,
}

impl<'a> dot::Labeller<'a, DotNode, DotEdge> for Graph {
    fn graph_id(&'a self) -> dot::Id<'a> {
        dot::Id::new("ParseTree Graph").unwrap()
    }
    fn node_id(&'a self, n: &DotNode) -> dot::Id<'a> {
        dot::Id::new(n.1.clone()).unwrap()
    }
    fn node_label(&self, n: &DotNode) -> dot::LabelText {
        let (_, label) = n.clone();
        dot::LabelText::HtmlStr(label.into())
    }
    fn edge_label(&self, _: &DotEdge) -> dot::LabelText {
        dot::LabelText::LabelStr("".into())
    }
}

impl<'a> dot::GraphWalk<'a, DotNode, DotEdge> for Graph {
    fn nodes(&'a self) -> dot::Nodes<'a, DotNode> {
        let nodes = self.nodes.clone().into_iter().enumerate().map(|(i, n)| (i, n)).collect();
        Cow::Owned(nodes)
    }
    fn edges(&'a self) -> dot::Edges<'a, DotEdge> {
        self.edges
            .iter()
            .map(|&(i, j)| ((i, self.nodes[i].clone()), (j, self.nodes[j].clone())))
            .collect()
    }
    fn source(&self, e: &DotEdge) -> DotNode {
        e.0.clone()
    }
    fn target(&self, e: &DotEdge) -> DotNode {
        e.1.clone()
    }
}
