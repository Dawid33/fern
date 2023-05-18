use crate::grammar::reader::TokenTypes;
use crate::grammar::{Associativity, Token};
use log::debug;
use std::collections::{BTreeSet, HashMap};
use std::fs::File;
use std::io::Write;

pub fn print_op_table(
    token_raw: &HashMap<Token, String>,
    token_reverse: &HashMap<String, (Token, TokenTypes)>,
    terminals: &Vec<Token>,
    op_table: &HashMap<Token, HashMap<Token, Associativity>>,
) {
    let mut sorted = Vec::new();
    for t in terminals {
        sorted.push(token_raw.get(&t).unwrap());
    }
    sorted.sort();
    let terminals: Vec<Token> = sorted
        .into_iter()
        .map(|n| token_reverse.get(n.as_str()).unwrap().0)
        .collect();

    let mut largest = 11;
    terminals.iter().for_each(|x| {
        let s_len = token_raw.get(x).unwrap().len();
        if s_len > largest {
            largest = s_len
        }
    });
    largest += 1;
    let mut builder = String::new();
    builder.push_str(format!("{:<l$}", "", l = largest).as_str());
    for row in &terminals {
        builder.push_str(format!("{:<l$}", token_raw.get(row).unwrap(), l = largest).as_str());
    }
    debug!("[OP TABLE] {}", builder);
    builder.clear();

    for row in &terminals {
        builder.push_str(format!("{:<l$}", token_raw.get(row).unwrap(), l = largest).as_str());
        let curr_row = op_table.get(row).unwrap();
        for col in &terminals {
            builder.push_str(format!("{:<l$}", format!("{:?}", curr_row.get(col).unwrap()), l = largest).as_str());
        }
        debug!("[OP TABLE] {}", builder);
        builder.clear();
    }
}


pub fn print_dict(name: &str, dict_rules: &HashMap<Vec<Token>, BTreeSet<Token>>, token_raw: &HashMap<Token, String>) {
    let mut f = File::create(name).unwrap();
    for (key, val) in dict_rules {
        let mut builder = String::new();
        builder.push_str("(");
        if !key.is_empty() {
            builder.push_str(format!("\'{}\'", token_raw.get(&key.get(0).unwrap()).unwrap()).as_str());
            if key.len() > 1 {
                for k in &key[1..key.len()] {
                    builder.push_str(", ");
                    builder.push_str(format!("\'{}\'", token_raw.get(&k).unwrap()).as_str());
                }
            } else {
                builder.push_str(",");
            }
            builder.push_str(") = [");

            let mut sorted = Vec::new();
            for x in val.iter() {
                sorted.push(token_raw.get(x).unwrap());
            }
            sorted.sort();

            let mut val_iter = sorted.iter();
            if val_iter.len() > 0 {
                builder.push_str(format!("\'{}\'", val_iter.next().unwrap()).as_str());
            }
            while let Some(t) = val_iter.next() {
                builder.push_str(", ");
                builder.push_str(format!("\'{}\'", t).as_str());
            }
        }
        builder.push_str("]\n");
        f.write(builder.as_bytes()).unwrap();
    }
}
