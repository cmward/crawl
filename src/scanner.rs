/*
* Largely ripped from Robert Nystrom's *Crafting Interpreters*
*/

// TODO: Actual error handling - indicate position of invalid input
// TODO: Tabs and newlines as tokens, since whitespace matters

const EOF_CHAR: char = '\0';

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Arrow,
    Calendar,
    ClearFact,
    ClearPersistentFact,
    Concat,
    Eof,
    FactTest,
    Int(i32),
    Minus,
    On,
    Plus,
    Procedure,
    Reminder,
    Roll,
    RollRange(i32, i32),
    RollValue(String),
    SetFact,
    SetPersistentFact,
    Str(String),
    Table,
    Tick,
}

#[derive(Debug)]
pub struct Scanner {
    source: Vec<char>,
    position: usize, // The character to be scanned
    line: usize,
    start: usize, // The start of the current lexeme
}

impl Scanner {
    pub fn new(source: Vec<char>) -> Self {
        Scanner {
            source,
            position: 0,
            line: 0,
            start: 0,
        }
    }

    pub fn tokens(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while !self.is_at_end() {
            self.start = self.position;
            let token = self.next_token();
            tokens.push(token);
        }
        tokens.push(Token::Eof);
        tokens
    }

    // TODO: -> Result<Token, ScannerError>
    fn next_token(&mut self) -> Token {
        loop {
            let ch = self.curr_char();
            self.advance();
            match ch {
                // Dice rolls, ranges & numbers
                n if n.is_numeric() => {
                    let mut next_ch = self.curr_char();
                    let mut is_dice_roll = false;
                    let mut is_roll_range = false;
                    while !self.is_at_end() {
                        match next_ch {
                            'd' => is_dice_roll = true,
                            '-' => is_roll_range = true,
                            nch if nch.is_numeric() => {}
                            _ => break,
                        }
                        self.advance();
                        next_ch = self.curr_char();
                    }
                    let lexeme = self.source[self.start..self.position]
                        .iter()
                        .collect::<String>();
                    match (is_dice_roll, is_roll_range) {
                        (true, false) => return Token::RollValue(lexeme),
                        (false, true) => {
                            let range_nums = lexeme.split('-').collect::<Vec<&str>>();
                            let range_min = range_nums
                                .first()
                                .expect("range min should be a value")
                                .parse::<i32>()
                                .expect("range min should be a number");
                            let range_max = range_nums
                                .last()
                                .expect("range max should be a value")
                                .parse::<i32>()
                                .expect("range max should be a number");
                            return Token::RollRange(range_min, range_max);
                        }
                        (false, false) => {
                            return Token::Int(lexeme.parse::<i32>().expect("should be a number"));
                        }
                        (true, true) => panic!("can't be a range and a roll"),
                    }
                }

                // Quoted text - Str
                '"' => {
                    while self.peek() != '"' && !self.is_at_end() {
                        self.advance();
                        if self.curr_char() == '\n' {
                            self.line += 1;
                        }
                    }
                    assert!(!self.is_at_end(), "expected closing '\"'");
                    // pass closing "
                    self.advance();
                    return Token::Str(
                        self.source[self.start + 1..self.position - 1]
                            .iter()
                            .collect(),
                    );
                }

                // Text - keywords
                c if c.is_alphabetic() => {
                    let mut next_ch = self.curr_char();
                    // function names can have hyphens in them
                    while !self.is_at_end() && (next_ch.is_alphabetic() || next_ch == '-') {
                        self.advance();
                        next_ch = self.curr_char();
                    }
                    let lexeme: String = self.source[self.start..self.position].iter().collect();
                    match Self::token_for_keyword(&lexeme) {
                        Some(token) => return token,
                        None => panic!("not a keyword"),
                    }
                }

                ' ' | '\t' => {
                    self.start = self.position;
                }

                '\n' => {
                    self.start = self.position;
                    self.line += 1;
                }

                '=' => {
                    if self.match_and_consume('>') {
                        return Token::Arrow;
                    }
                    panic!("expected '>' after '='");
                }

                '+' => return Token::Plus,

                '-' => return Token::Minus,

                x => {
                    panic!("unexpected character {}", x);
                }
            }
        }
    }

    fn token_for_keyword(lexeme: &str) -> Option<Token> {
        match lexeme {
            "calendar" => Some(Token::Calendar),
            "clear-fact" => Some(Token::ClearFact),
            "clear-persistent-fact" => Some(Token::ClearPersistentFact),
            "fact?" => Some(Token::FactTest),
            "on" => Some(Token::On),
            "procedure" => Some(Token::Procedure),
            "reminder" => Some(Token::Reminder),
            "roll" => Some(Token::Roll),
            "set-fact" => Some(Token::SetFact),
            "set-persistent-fact" => Some(Token::SetPersistentFact),
            "table" => Some(Token::Table),
            "tick" => Some(Token::Tick),
            _ => None,
        }
    }

    fn curr_char(&mut self) -> char {
        if self.is_at_end() {
            return EOF_CHAR;
        }
        self.source[self.position]
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            return EOF_CHAR;
        }
        self.source[self.position]
    }

    fn peek_next(&self) -> char {
        if self.position + 1 >= self.source.len() {
            return EOF_CHAR;
        }
        self.source[self.position + 1]
    }

    fn match_and_consume(&mut self, ch: char) -> bool {
        if self.peek() == ch {
            self.advance();
            true
        } else {
            false
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.source.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_if_then() {
        let source = "\"Hi\" => 5".chars().collect();
        let mut scanner = Scanner::new(source);
        assert_eq!(
            scanner.tokens(),
            vec![
                Token::Str(String::from("Hi")),
                Token::Arrow,
                Token::Int(5),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn scan_proc_def() {
        let source = "procedure \"proc\"".chars().collect();
        let mut scanner = Scanner::new(source);
        assert_eq!(
            scanner.tokens(),
            vec![
                Token::Procedure,
                Token::Str(String::from("proc")),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn scan_roll_range() {
        let source = "roll 2-10".chars().collect();
        let mut scanner = Scanner::new(source);
        assert_eq!(
            scanner.tokens(),
            vec![Token::Roll, Token::RollRange(2, 10), Token::Eof]
        );
    }

    #[test]
    fn scan_expr() {
        let source = "roll 1-3 on 1d6 + 1 => set-fact \"party is lost\""
            .chars()
            .collect();
        let mut scanner = Scanner::new(source);
        assert_eq!(
            scanner.tokens(),
            vec![
                Token::Roll,
                Token::RollRange(1, 3),
                Token::On,
                Token::RollValue(String::from("1d6")),
                Token::Plus,
                Token::Int(1),
                Token::Arrow,
                Token::SetFact,
                Token::Str(String::from("party is lost")),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn scan_concat() {
        let source = "set-fact \"weather is \" + roll on table \"weather\""
            .chars()
            .collect();
        let mut scanner = Scanner::new(source);
        assert_eq!(
            scanner.tokens(),
            vec![
                Token::SetFact,
                Token::Str(String::from("weather is ")),
                Token::Plus,
                Token::Roll,
                Token::On,
                Token::Table,
                Token::Str(String::from("weather")),
                Token::Eof,
            ]
        );
    }

    #[test]
    #[should_panic(expected = "expected '>' after '='")]
    fn incomplete_arrow() {
        let source = "= 5".chars().collect();
        let mut scanner = Scanner::new(source);
        scanner.tokens();
    }

    #[test]
    #[should_panic(expected = "expected closing '\"'")]
    fn unterminated_string() {
        let source = "\"Unterminated string".chars().collect();
        let mut scanner = Scanner::new(source);
        scanner.tokens();
    }
}
