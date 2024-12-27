use crawl::DiceRoll;

fn main() {
    let roll = "2d6 - 3".parse::<DiceRoll>().unwrap();
    println!("{:?}", roll);
    println!("{}", roll);
    let result = roll.roll();
    println!("{:?}", result);
    println!("{} = {}", roll, result);
}
