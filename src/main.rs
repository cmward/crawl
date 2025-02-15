use crawl::lang::Crawl;
use std::io::{self, Write};

fn main() {
    // TODO: no args -> repl, one arg -> execute file

    // let input = "roll 1-3 on 1d6\nreminder \"hi :D\"\n".to_string();
    let input = "if roll 1-3 on 1d6 => reminder \"hi :D\"\n".to_string();
    /*
    print!(">> ");
    std::io::stdout().flush().unwrap();

    let mut input = String::new();

    io::stdin()
        .read_line(&mut input)
        .expect("failed to read line");
    */

    let crawl = Crawl::new();
    crawl.execute(&input);
}
