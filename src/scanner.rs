/*
* Largely ripped from Robert Nystrom's *Crafting Interpreters*
*/

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
    Hyphen,
    Int(i32),
    On,
    Procedure,
    Reminder,
    Roll,
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
        dbg!(&self.source);
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
            match dbg!(ch) {
                ' ' | '\t' => {
                    self.advance();
                    self.start = self.position;
                }
                '\n' => {
                    self.advance();
                    self.start = self.position;
                    self.line += 1;
                }
                '=' => {
                    if self.match_lookahead_and_consume('>') {
                        return Token::Arrow;
                    }
                    panic!("expected '>' after '='");
                }
                '-' => return Token::Hyphen,
                // Quoted text - Str
                '"' => {
                    self.advance();
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
                // Numbers
                n if n.is_numeric() => {
                    let mut next_ch = self.curr_char();
                    while !self.is_at_end() && next_ch.is_numeric() {
                        self.advance();
                        next_ch = self.curr_char();
                    }
                    return Token::Int(
                        self.source[self.start..self.position]
                            .iter()
                            .collect::<String>()
                            .parse::<i32>()
                            .expect("should be a number"),
                    );
                }
                // Text - keywords
                c if c.is_alphabetic() => {
                    let mut next_ch = self.curr_char();
                    while !self.is_at_end() && next_ch.is_alphabetic() {
                        self.advance();
                        next_ch = self.curr_char();
                    }
                    let lexeme: String = self.source[self.start..self.position].iter().collect();
                    match Self::token_for_keyword(&lexeme) {
                        Some(token) => return token,
                        None => panic!("not a keyword"),
                    }
                }
                x => {
                    dbg!(x);
                    todo!();
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

    fn match_lookahead_and_consume(&mut self, ch: char) -> bool {
        if self.peek_next() == ch {
            self.advance();
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
    fn scan() {
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
