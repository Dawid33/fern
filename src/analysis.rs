use std::collections::{HashMap, VecDeque};

use log::{debug, error, info, warn};

use crate::{ir::SymbolData, parser::fern_ast::AstNode};

#[derive(Clone, Debug)]
struct ScopeTreeNode {
    pub tbl: HashMap<String, ()>,
    pub nodes: Vec<ScopeTreeNode>,
}

impl ScopeTreeNode {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            tbl: HashMap::new(),
        }
    }
}

pub fn check_used_before_declared(root: Box<AstNode>) -> Vec<Box<AstNode>> {
    let root = if let AstNode::Module(list) = *root {
        list
    } else {
        panic!("Only works with modules")
    };
    let mut scopes: Vec<(HashMap<String, ()>, VecDeque<AstNode>)> = Vec::from(&[(HashMap::new(), [*root].into_iter().collect())]);
    let mut final_scopes = Vec::new();

    let new_scopes = |scopes: &mut Vec<(HashMap<String, ()>, VecDeque<AstNode>)>,
                      old_tbl: HashMap<String, ()>,
                      old_nodes: VecDeque<AstNode>,
                      nodes: VecDeque<AstNode>,
                      final_scopes: &mut Vec<ScopeTreeNode>| {
        info!("Creating new scope.");
        scopes.push((old_tbl, old_nodes));
        scopes.push((HashMap::new(), nodes.into()));
        final_scopes.push(ScopeTreeNode::new());
    };

    pub fn check_exists(scopes: &Vec<(HashMap<String, ()>, VecDeque<AstNode>)>, local_tbl: &HashMap<String, ()>, name: &String) -> bool {
        if local_tbl.contains_key(name) {
            return true;
        }
        for (tbl, _) in scopes.iter().rev() {
            if tbl.contains_key(name) {
                return true;
            }
        }

        false
    }

    'outer: while let Some((mut local_tbl, mut s)) = scopes.pop() {
        while let Some(n) = s.pop_front() {
            match n {
                AstNode::Binary(_, _, _) => {}
                AstNode::Unary(_, _) => {}
                AstNode::Number(_) => {}
                AstNode::String(_) => {}
                AstNode::Name(_) => {}
                AstNode::ExprList(_) => {}
                AstNode::Assign(name, _expr) => {
                    if let AstNode::Name(name) = *name {
                        if !check_exists(&scopes, &local_tbl, &name) {
                            warn!("Identifier {} is used but not previously declared.", name);
                        }
                    }
                }
                AstNode::Let(left, _type, _right) => match *left {
                    AstNode::Name(name) => {
                        info!("Inserting {} into symbol table.", name);
                        if check_exists(&scopes, &local_tbl, &name) {
                            warn!("Identifier {} already exists in super scope.", name);
                        }
                        local_tbl.insert(name, ());
                    }
                    _ => error!("Wrong ast node type on left side of let statement."),
                },
                AstNode::Return(_) => {}
                AstNode::Module(list) => {
                    if let AstNode::StatList(_) = *list {
                        s.push_back(*list);
                    }
                }
                AstNode::StatList(nodes) => {
                    info!("Inside statlist");
                    new_scopes(&mut scopes, local_tbl, s, nodes, &mut final_scopes);
                    continue 'outer;
                }
                AstNode::FunctionCall(_, _) => {}
                AstNode::Function(_, _, body) => {
                    info!("Inside function");
                    if let Some(list) = body {
                        if let AstNode::StatList(nodes) = *list {
                            new_scopes(&mut scopes, local_tbl, s, nodes, &mut final_scopes);
                            continue 'outer;
                        } else {
                            error!("big bad.");
                        }
                    }
                }
                AstNode::If(_expr, body, _next) => {
                    if let Some(stat_list) = body {
                        if let AstNode::StatList(nodes) = *stat_list {
                            new_scopes(&mut scopes, local_tbl, s, nodes, &mut final_scopes);
                            continue 'outer;
                        }
                    }
                }
                AstNode::ExprThen(_, _) => {}
                AstNode::ElseIf(_, _, _) => {}
                AstNode::Else(_) => {}
                AstNode::For(_, _, _) => {}
                AstNode::While(_, _) => {}
            }
        }

        if final_scopes.len() > 1 {
            let mut last = final_scopes.pop().unwrap();
            let second_last = final_scopes.last_mut().unwrap();
            last.tbl = local_tbl;
            second_last.nodes.push(last);
        } else if scopes.len() > 0 {
            let last = final_scopes.last_mut().unwrap();
            last.tbl = local_tbl;
        }
    }

    println!("{:?}", final_scopes);

    return Vec::new();
}
