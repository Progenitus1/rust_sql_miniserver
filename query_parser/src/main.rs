mod parser;

use std::collections::HashMap;

use parser::expression_tree::parse_tree;
use parser::expression_tree_eval::evaluate_binary_node;
use parser::lexer::lex;
use parser::query_parser::parse;

fn main() {
    dbg!(parse("SELECT *, 1, id FROM my_table WHERE x = 2").unwrap());
    dbg!(parse(stringify!(insert into my_table values 1,3,4.300)).unwrap());

    let tree = parse_tree(lex("2 + 2 = 4").unwrap()).unwrap().unwrap();
    dbg!(evaluate_binary_node(&tree, &HashMap::new()).unwrap());
}
