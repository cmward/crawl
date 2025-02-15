use crawl::lang::Crawl;
use std::io::{self, Write};

fn main() {
    let input = "roll 1-3 on 1d6\nreminder \"hi :D\"\n";
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
