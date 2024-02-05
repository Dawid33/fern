use crate::grammar::opg::OpGrammar;
use crate::parser::{Node, ParseTree};
use std::cmp::max;

// fn json() -> Result<(), Box<dyn Error>> {
//     let mut now = Instant::now();
//     let mut raw = RawGrammar::from("data/grammar/json.g")?;
//     raw.delete_repeated_rhs()?;
//     let grammar = OpGrammar::new(raw)?;
//     grammar.to_file("data/grammar/json-fnf.g");
//     info!("Total Time to get grammar : {:?}", now.elapsed());
//     now = Instant::now();

//     let tokens: LinkedList<Vec<(Token, JsonData)>> = {
//         let file = File::open("data/json/100KB.json")?;
//         let mmap: memmap::Mmap = unsafe { MmapOptions::new().map(&file)? };
//         thread::scope(|s| {
//             let mut lexer: ParallelLexer<JsonLexerState, JsonLexer, JsonData> =
//                 ParallelLexer::new(&grammar, s, 16, &[JsonLexerState::Start, JsonLexerState::InString], JsonLexerState::Start);
//             let batch = lexer.new_batch();
//             lexer.add_to_batch(&batch, &mmap[..], 0);
//             let tokens = lexer.collect_batch(batch);
//             lexer.kill();
//             tokens
//         })
//     };

//     info!("Total Time to lex: {:?}", now.elapsed());
//     now = Instant::now();

//     // let (tree, time): (JsonParseTree, Duration) = {
//     //     let mut parser = ParallelParser::new(grammar.clone(), 1);
//     //     parser.parse(tokens);
//     //     parser.parse(LinkedList::from([vec![(grammar.delim, JsonData::NoData)]]));
//     //     let time = parser.time_spent_rule_searching.clone();
//     //     (parser.collect_parse_tree().unwrap(), time)
//     // };

//     // tree.print();
//     // info!("Total Time to parse: {:?}", now.elapsed());
//     // info!("└─Total Time spent rule-searching: {:?}", time);

//     // now = Instant::now();
//     // info!("Total Time to transform ParseTree -> AST Conversion: {:?}", now.elapsed());
//     Ok(())
// }

#[derive(Debug, Clone)]
pub enum JsonValue {
    Null,
    String(String),
    Number(Number),
    Boolean(bool),
    Object(Object),
    Array(Vec<JsonValue>),
}

#[derive(Debug, Clone)]
pub struct Number {}

#[derive(Debug, Clone)]
pub struct Object {}

pub struct JsonParseTree {}
impl ParseTree for JsonParseTree {
    fn new(_root: Node, _g: OpGrammar) -> Self {
        Self {}
    }

    fn print(&self) {}
}
