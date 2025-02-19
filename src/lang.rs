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

        println!("{toks:?}\n");

        let ast = Parser::new(toks)
            .parse()
            .into_iter()
            .map(|node| node.unwrap())
            .collect();

        println!("{ast:?}\n");

        let mut interpreter = Interpreter::new();
        let records: Vec<StatementRecord> = interpreter
            .interpret(ast)
            .into_iter()
            .map(|record| record.unwrap())
            .collect();

        println!("{records:?}\n");

        println!("{:?}", interpreter.local_facts);
    }
}
