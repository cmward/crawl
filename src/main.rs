use crawl::dice::DiceRoll;
use crawl::scanner::Scanner;

fn main() {
    let roll = "2d6 + 1".parse::<DiceRoll>().unwrap();
    println!("{roll:?}");
    println!("{roll}");
    let result = roll.roll();
    println!("{result:?}");
    println!("{roll} = {result}");

    let mut scanner = Scanner::new("roll on".chars().collect());
    for token in scanner.tokens() {
        println!("TOKEN: {:?}", token);
    }

    let mut scanner = Scanner::new("=".chars().collect());
    for token in scanner.tokens() {
        println!("TOKEN: {:?}", token);
    }
}
