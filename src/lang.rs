use crate::interpreter::{Interpreter, StatementRecord};
use crate::parser::Parser;
use crate::scanner::Scanner;

pub struct Crawl;

impl Crawl {
    pub fn new() -> Self {
        Crawl
    }

    pub fn execute(&self, source: &str) {
        let toks = Scanner::new(source.chars().collect())
            .tokens()
            .into_iter()
            .map(|tok| tok.unwrap())
            .collect();

        println!("{toks:?}");

        let ast = Parser::new(toks)
            .parse()
            .into_iter()
            .map(|node| node.unwrap())
            .collect();

        println!("{ast:?}");

        let records: Vec<StatementRecord> = Interpreter::new()
            .interpret(ast)
            .into_iter()
            .map(|record| record.unwrap())
            .collect();

        println!("{records:?}");
    }
}
