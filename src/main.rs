use crawl::lang::Crawl;
use std::{
    env,
    error::Error,
    ffi::OsString,
    fs,
    io::{self, Write},
    process::exit,
};

fn main() -> Result<(), Box<dyn Error>> {
    match env::args_os().nth(1) {
        Some(filepath) => execute_file(filepath),
        None => repl(),
    }
}

fn execute_file(filepath: OsString) -> Result<(), Box<dyn Error>> {
    let input = fs::read_to_string(filepath)?;
    let crawl = Crawl::new();
    crawl.execute(&input);
    Ok(())
}

fn repl() -> Result<(), Box<dyn Error>> {
    ctrlc::set_handler(move || exit(1)).expect("failed to set ctrlc handler");

    loop {
        print!(">> ");
        std::io::stdout().flush().unwrap();

        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("failed to read line");

        let crawl = Crawl::new();
        crawl.execute(&input);
    }
}
