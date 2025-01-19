use crawl::dice::DiceRoll;
use crawl::scanner::Scanner;

fn main() {
    let roll = "2d6 + 1".parse::<DiceRoll>().unwrap();
    println!("{roll:?}");
    println!("{roll}");
    let result = roll.roll();
    println!("{result:?}");
    println!("{roll} = {result}");

    let mut scanner = Scanner::new(dbg!("roll on").chars().collect());
    for token in scanner.tokens() {
        println!("{token:?}");
    }

    let mut scanner = Scanner::new(dbg!("= hello yup - 2d + 100c").chars().collect());
    for token in scanner.tokens() {
        println!("{token:?}");
    }
}
