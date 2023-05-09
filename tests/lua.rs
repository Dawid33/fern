extern crate core;

use core::grammar::OpGrammar;
use core::lex_lua;
use log::info;
use std::error::Error;
use std::fs::File;

fn get_grammar() -> OpGrammar {
    let grammar = match File::open(".lua-grammar") {
        Ok(f) => {
            info!("Using cached grammar from file : .lua-grammar");
            let grammar = ciborium::de::from_reader::<'_, OpGrammar, _>(f).unwrap();
            grammar
        }
        Err(_) => {
            info!("Generating grammar from scratch...");
            let grammar = OpGrammar::from("data/grammar/lua.g");
            // let f = File::create(".lua-grammar").unwrap();
            // info!("Grammar saved to .lua-grammar");
            // ciborium::ser::into_writer(&grammar, f).unwrap();
            grammar
        }
    };
    grammar
}

fn test_lua(input: &str, expected: Vec<&str>) {
    let g = get_grammar();
    let result = lex_lua(input, &g).unwrap();
    let mut size = 0;
    for list in result {
        size += list.len();
        for (i, t) in list.iter().enumerate() {
            assert_eq!(
                *t,
                g.token_reverse.get(*expected.get(i).unwrap()).unwrap().0,
                "Recieved {}, expected {}.",
                g.token_raw.get(t).unwrap(),
                expected.get(i).unwrap()
            );
        }
    }
    assert_eq!(
        size,
        expected.len(),
        "Number of recieved tokens ({}) doesn't equal number of expected tokens ({}).",
        size,
        expected.len()
    );
}

#[test]
fn test_simple_stmt() {
    test_lua("local x = 0;", vec!["LOCAL", "NAME", "XEQ", "NUMBER", "SEMI"]);
}

#[test]
fn test_for() {
    test_lua(
        "for c = 0, 323 do R[c] = {} end",
        vec![
            "FOR", "NAME", "XEQ", "NUMBER", "COMMA", "NUMBER", "DO", "NAME", "LBRACK", "NAME", "RBRACK", "XEQ",
            "LBRACE", "RBRACE", "END",
        ],
    );
}
