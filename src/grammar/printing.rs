use crate::grammar::reader::TokenTypes;
use crate::grammar::{Associativity, Token};
use log::debug;
use std::collections::HashMap;

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
