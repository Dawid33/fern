use crate::grammar::Associativity::{Equal, Left, Right};
use std::collections::hash_map::HashMap;
use std::collections::HashSet;

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
#[allow(unused)]
pub enum Token {
    Delim,
    Start,
    Object,
    Members,
    Pair,
    String,
    Value,
    Array,
    Elements,
    Chars,
    Char,

    RightCurly,
    LeftCurly,
    Colon,
    Number,
    Bool,
    Quote,
    LeftSqrBracket,
    RightSqrBracket,
    Comma,
    Character,
}

pub struct Rule {
    pub left: Token,
    pub right: Vec<Token>,
}

#[derive(Clone, Debug, Copy)]
#[allow(unused)]
enum Associativity {
    None,
    Left,
    Right,
    Equal,
    Undefined,
}

#[allow(unused)]
pub struct Grammar {
    non_terminals: Vec<Token>,
    terminals: Vec<Token>,
    delim: Token,
    axiom: Token,
    inverse_rewrite_rules: HashMap<Token, Vec<Token>>,
    rules: Vec<Rule>,
    op_table: HashMap<Token, HashMap<Token, Associativity>>,
}

#[allow(unused)]
impl Grammar {
    pub fn new(
        rules: Vec<Rule>,
        terminals: Vec<Token>,
        non_terminals: Vec<Token>,
        axiom: Token,
        delim: Token,
    ) -> Grammar {
        let mut inverse_rewrite_rules: HashMap<Token, Vec<Token>> = HashMap::new();
        let mut op_table: HashMap<Token, HashMap<Token, Associativity>> = HashMap::new();

        // Create re-write rules
        // TODO : Figure out how this actually works.
        let mut rewrite_rules: HashMap<Token, Vec<Token>> = HashMap::new();
        for t in &non_terminals {
            rewrite_rules.insert(*t, Vec::new());
        }
        let mut modified = true;
        while modified {
            modified = false;
            for r in &rules {
                let token = r.right.get(0).unwrap();
                if r.right.len() != 1 || terminals.contains(token) {
                    continue;
                }
                if !rewrite_rules.get_mut(&r.left).unwrap().contains(token) {
                    modified = true;
                    rewrite_rules.get_mut(&r.left).unwrap().push(*token);
                } else {
                    for ttoken in rewrite_rules.get(token).unwrap().clone() {
                        if !rewrite_rules.get(&r.left).unwrap().contains(&ttoken) {
                            modified = true;
                            rewrite_rules.get_mut(&r.left).unwrap().push(ttoken);
                        }
                    }
                }
            }
        }

        // Create inverse rewrite rules
        // TODO: Figure out what this is.
        for t in &non_terminals {
            inverse_rewrite_rules.insert(*t, vec![*t]);
        }
        for t in &non_terminals {
            for t1 in rewrite_rules.get(t).unwrap() {
                inverse_rewrite_rules.get_mut(t1).unwrap().push(*t);
            }
        }

        let mut first_ops: HashMap<Token, HashSet<Token>> = HashMap::new();
        let mut last_ops: HashMap<Token, HashSet<Token>> = HashMap::new();

        for r in &rules {
            if non_terminals.contains(&r.left) {
                if r.right.len() > 0 {
                    for s in &r.right {
                        if terminals.contains(&s) {
                            if !first_ops.contains_key(&r.left) {
                                first_ops.insert(r.left, HashSet::from([*s]));
                            } else {
                                first_ops.get_mut(&r.left).unwrap().insert(*s);
                            }
                            break;
                        }
                    }

                    // Possible error, check later
                    for i in (0..r.right.len()).rev() {
                        if terminals.contains(&r.right[i]) {
                            if !last_ops.contains_key(&r.left) {
                                last_ops.insert(r.left, HashSet::from([r.right[i]]));
                            } else {
                                last_ops.get_mut(&r.left).unwrap().insert(r.right[i]);
                            }
                            break;
                        }
                    }
                }
            }
        }

        let mut did_something: bool;
        loop {
            did_something = false;
            for r in &rules {
                if non_terminals.contains(&r.left) {
                    if r.right.len() > 0 {
                        if non_terminals.contains(&r.right[0]) {
                            if first_ops.contains_key(&r.right[0]) {
                                let bs = first_ops.get_mut(&r.right[0]).unwrap().clone();
                                if !first_ops.contains_key(&r.left) {
                                    did_something = true;
                                    first_ops
                                        .insert(r.left, HashSet::from_iter(bs.clone().into_iter()));
                                } else if !first_ops.get(&r.left).unwrap().is_superset(&bs) {
                                    did_something = true;
                                    for x in bs {
                                        first_ops.get_mut(&r.left).unwrap().insert(x);
                                    }
                                }
                            }
                        }

                        if non_terminals.contains(&r.right[r.right.len() - 1]) {
                            if last_ops.contains_key(&r.right[r.right.len() - 1]) {
                                let bs = last_ops.get(&r.right[r.right.len() - 1]).unwrap().clone();
                                if !last_ops.contains_key(&r.left) {
                                    did_something = true;
                                    last_ops
                                        .insert(r.left, HashSet::from_iter(bs.clone().into_iter()));
                                } else if !last_ops.get(&r.left).unwrap().is_superset(&bs) {
                                    did_something = true;
                                    for x in bs {
                                        last_ops.get_mut(&r.left).unwrap().insert(x);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if did_something {
                continue;
            } else {
                break;
            }
        }

        println!("FIRST OP");
        for row in first_ops.keys() {
            println!("{:?} : {:?}", row, first_ops.get(row));
        }
        println!();

        println!("LAST OP");
        for row in last_ops.keys() {
            println!("{:?} : {:?}", row, last_ops.get(row));
        }
        println!();

        let mut template: HashMap<Token, Associativity> = HashMap::new();
        for t in &terminals {
            template.insert(*t, Associativity::None);
        }

        for t in &terminals {
            op_table.insert(*t, template.clone());
        }

        for r in &rules {
            for i in 0..r.right.len() {
                if i + 1 < r.right.len() {
                    if terminals.contains(r.right.get(i).unwrap())
                        && terminals.contains(r.right.get(i + 1).unwrap())
                    {
                        op_table
                            .get_mut(r.right.get(i).unwrap())
                            .unwrap()
                            .insert(*r.right.get(i + 1).unwrap(), Equal);
                    }
                    if terminals.contains(r.right.get(i).unwrap())
                        && non_terminals.contains(r.right.get(i + 1).unwrap())
                    {
                        if first_ops.contains_key(r.right.get(i + 1).unwrap()) {
                            let first_op_a = first_ops.get(r.right.get(i + 1).unwrap()).unwrap();
                            for q2 in first_op_a {
                                op_table
                                    .get_mut(r.right.get(i).unwrap())
                                    .unwrap()
                                    .insert(*q2, Left);
                            }
                        }
                    }
                    if non_terminals.contains(r.right.get(i).unwrap())
                        && terminals.contains(r.right.get(i + 1).unwrap())
                    {
                        if last_ops.contains_key(r.right.get(i).unwrap()) {
                            let last_op_a = last_ops.get(r.right.get(i).unwrap()).unwrap();
                            for q2 in last_op_a {
                                op_table
                                    .get_mut(q2)
                                    .unwrap()
                                    .insert(*r.right.get(i + 1).unwrap(), Right);
                            }
                        }
                    }
                    if i + 2 < r.right.len() {
                        if terminals.contains(r.right.get(i).unwrap())
                            && non_terminals.contains(r.right.get(i + 1).unwrap())
                            && terminals.contains(r.right.get(i + 2).unwrap())
                        {
                            op_table
                                .get_mut(r.right.get(i).unwrap())
                                .unwrap()
                                .insert(*r.right.get(i + 2).unwrap(), Equal);
                        }
                    }
                }
            }
        }

        print!("{:<16}", "");
        for row in &terminals {
            print!("{:16}", format!("{:?}", row));
        }
        println!();

        for row in &terminals {
            print!("{:16}", format!("{:?}", row));
            let curr_row = op_table.get(row).unwrap();
            for col in &terminals {
                print!("{:16}", format!("{:?}", curr_row.get(col).unwrap()));
            }
            println!();
        }
        println!();

        Grammar {
            rules,
            terminals,
            non_terminals,
            axiom,
            delim,
            inverse_rewrite_rules,
            op_table,
        }
    }

    pub fn json_grammar() -> Grammar {
        use crate::grammar::Token::{
            Array, Bool, Character, Chars, Colon, Comma, Delim, Elements, LeftCurly,
            LeftSqrBracket, Members, Number, Object, Pair, Quote, RightCurly, RightSqrBracket,
            String, Value,
        };
        let terminals: Vec<Token> = vec![
            LeftCurly,
            RightCurly,
            Colon,
            Comma,
            Number,
            Bool,
            Quote,
            Character,
            LeftSqrBracket,
            RightSqrBracket,
        ];
        let non_terminals: Vec<Token> =
            vec![Object, Members, Pair, String, Value, Array, Elements, Chars];
        let rules: Vec<Rule> = vec![
            Rule {
                left: Object,
                right: vec![LeftCurly, Members, RightCurly],
            },
            Rule {
                left: Object,
                right: vec![LeftCurly, RightCurly],
            },
            Rule {
                left: Members,
                right: vec![Pair],
            },
            Rule {
                left: Members,
                right: vec![Pair, Comma, Members],
            },
            Rule {
                left: Pair,
                right: vec![String, Colon, Value],
            },
            Rule {
                left: Value,
                right: vec![String],
            },
            Rule {
                left: Value,
                right: vec![Number],
            },
            Rule {
                left: Value,
                right: vec![Object],
            },
            Rule {
                left: Value,
                right: vec![Array],
            },
            Rule {
                left: Value,
                right: vec![Bool],
            },
            Rule {
                left: String,
                right: vec![Quote, Quote],
            },
            Rule {
                left: String,
                right: vec![Quote, Chars, Quote],
            },
            Rule {
                left: Array,
                right: vec![LeftSqrBracket, RightSqrBracket],
            },
            Rule {
                left: Array,
                right: vec![LeftSqrBracket, Elements, RightSqrBracket],
            },
            Rule {
                left: Elements,
                right: vec![Value],
            },
            Rule {
                left: Elements,
                right: vec![Value, Comma, Elements],
            },
            Rule {
                left: Chars,
                right: vec![Character],
            },
            Rule {
                left: Chars,
                right: vec![Character, Chars],
            },
        ];

        Grammar::new(rules, terminals, non_terminals, Object, Delim)
    }

    #[allow(unused)]
    fn get_precedence(&self, left: Token, right: Token) -> Associativity {
        return self
            .op_table
            .get(&left)
            .unwrap()
            .get(&right)
            .unwrap()
            .clone();
    }
}
