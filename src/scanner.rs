/*
* Largely ripped from Robert Nystrom's *Crafting Interpreters*
*/

use crate::error::CrawlError;

const EOF_CHAR: char = '\0';

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Arrow,
    ClearFact,
    ClearPersistentFact,
    Concat,
    End,
    Eof,
    FactTest,
    Identifier(String),
    If,
    Indent,
    Minus,
    Newline,
    Num(i32),
    NumRange(i32, i32),
    On,
    Plus,
    Procedure,
    Reminder,
    Roll,
    RollSpecifier(String),
    SetFact,
    SetPersistentFact,
    Str(String),
    SwapFact,
    SwapPersistentFact,
    Table,
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

    pub fn tokens(&mut self) -> Vec<Result<Token, CrawlError>> {
        if self.is_at_end() {
            return Vec::new();
        }

        let mut toks = Vec::new();
        while !self.is_at_end() {
            self.start = self.position;
            toks.push(self.next_token());
        }
        toks.push(Ok(Token::Eof));

        toks
    }

    fn next_token(&mut self) -> Result<Token, CrawlError> {
        loop {
            let ch = self.curr_char();
            self.advance();
            match ch {
                // Dice rolls, ranges & numbers
                n if n.is_numeric() => return self.scan_numeric(),

                // Quoted text - Str
                '"' => return self.scan_str(),

                // Text - keywords
                c if c.is_alphabetic() => return self.scan_symbol(),

                // This is the only reason this needs to be wrapped in a loop
                ' ' => self.start = self.position,

                '\t' => return Ok(Token::Indent),

                '\n' => {
                    self.line += 1;
                    return Ok(Token::Newline);
                }

                '=' => {
                    if self.match_and_consume('>') {
                        return Ok(Token::Arrow);
                    }
                    return Err(CrawlError::ScannerError {
                        position: self.position,
                        line: self.line,
                        lexeme: self.source[self.start..self.position].iter().collect(),
                        reason: "expected '>' after '='".into(),
                    });
                }

                '+' => return Ok(Token::Plus),

                '-' => return Ok(Token::Minus),

                c => {
                    return Err(CrawlError::ScannerError {
                        position: self.position,
                        line: self.line,
                        lexeme: c.into(),
                        reason: "unexpected character".into(),
                    })
                }
            }
        }
    }

    fn scan_numeric(&mut self) -> Result<Token, CrawlError> {
        let mut next_ch = self.curr_char();
        let mut is_dice_roll = false;
        let mut is_roll_range = false;
        while !self.is_at_end() {
            match next_ch {
                'd' => {
                    if !self.peek_next().is_numeric() {
                        return Err(CrawlError::ScannerError {
                            position: self.position,
                            line: self.line,
                            lexeme: self.source[self.start..self.position]
                                .iter()
                                .collect::<String>(),
                            reason: "roll specifier must be NUMBER 'd' NUMBER".into(),
                        });
                    }
                    is_dice_roll = true;
                }
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
            (true, false) => Ok(Token::RollSpecifier(lexeme)),
            (false, true) => {
                let range_nums = lexeme.split('-').collect::<Vec<&str>>();
                // TODO: produce ScannerErrors here
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
                Ok(Token::NumRange(range_min, range_max))
            }
            (false, false) => Ok(Token::Num(
                lexeme.parse::<i32>().expect("should be a number"),
            )),
            (true, true) => Err(CrawlError::ScannerError {
                position: self.position,
                line: self.line,
                lexeme,
                reason: "can't be a dice roll and dice range".into(),
            }),
        }
    }

    fn scan_str(&mut self) -> Result<Token, CrawlError> {
        while self.peek() != '"' && !self.is_at_end() {
            self.advance();
            if self.curr_char() == '\n' {
                self.line += 1;
            }
        }
        if self.is_at_end() {
            return Err(CrawlError::ScannerError {
                position: self.position,
                line: self.line,
                lexeme: self.source[self.start..self.position].iter().collect(),
                reason: "unterminated string, expected closing '\"'".into(),
            });
        }
        // pass closing "
        self.advance();
        Ok(Token::Str(
            self.source[self.start + 1..self.position - 1]
                .iter()
                .collect(),
        ))
    }

    fn scan_symbol(&mut self) -> Result<Token, CrawlError> {
        let mut next_ch = self.curr_char();
        // function names can have hyphens in them
        while !self.is_at_end() && (next_ch.is_alphabetic() || next_ch == '-') {
            self.advance();
            next_ch = self.curr_char();
        }
        let lexeme: String = self.source[self.start..self.position].iter().collect();
        match Self::token_for_keyword(&lexeme) {
            Some(token) => Ok(token),
            None => Ok(Token::Identifier(lexeme)),
        }
    }

    fn token_for_keyword(lexeme: &str) -> Option<Token> {
        match lexeme {
            "clear-fact" => Some(Token::ClearFact),
            "clear-persistent-fact" => Some(Token::ClearPersistentFact),
            "end" => Some(Token::End),
            "fact?" => Some(Token::FactTest),
            "if" => Some(Token::If),
            "on" => Some(Token::On),
            "procedure" => Some(Token::Procedure),
            "reminder" => Some(Token::Reminder),
            "roll" => Some(Token::Roll),
            "set-fact" => Some(Token::SetFact),
            "set-persistent-fact" => Some(Token::SetPersistentFact),
            "swap-fact" => Some(Token::SwapFact),
            "swap-persistent-fact" => Some(Token::SwapPersistentFact),
            "table" => Some(Token::Table),
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
        let source = "if \"Hi\" => 5".chars().collect();
        let mut scanner = Scanner::new(source);
        let toks: Vec<Token> = scanner.tokens().into_iter().map(|t| t.unwrap()).collect();
        assert_eq!(
            toks,
            vec![
                Token::If,
                Token::Str("Hi".into()),
                Token::Arrow,
                Token::Num(5),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn scan_proc_decl() {
        let source = "procedure proc".chars().collect();
        let mut scanner = Scanner::new(source);
        let toks: Vec<Token> = scanner.tokens().into_iter().map(|t| t.unwrap()).collect();
        assert_eq!(
            toks,
            vec![
                Token::Procedure,
                Token::Identifier("proc".into()),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn scan_proc_call() {
        let source = "proc".chars().collect();
        let mut scanner = Scanner::new(source);
        let toks: Vec<Token> = scanner.tokens().into_iter().map(|t| t.unwrap()).collect();
        assert_eq!(toks, vec![Token::Identifier("proc".into()), Token::Eof]);
    }

    #[test]
    fn scan_roll_range() {
        let source = "roll 2-10".chars().collect();
        let mut scanner = Scanner::new(source);
        let toks: Vec<Token> = scanner.tokens().into_iter().map(|t| t.unwrap()).collect();
        assert_eq!(toks, vec![Token::Roll, Token::NumRange(2, 10), Token::Eof]);
    }

    #[test]
    fn scan_expr() {
        let source = "roll 1-3 on 1d6 + 1 => set-fact \"party is lost\""
            .chars()
            .collect();
        let mut scanner = Scanner::new(source);
        let toks: Vec<Token> = scanner.tokens().into_iter().map(|t| t.unwrap()).collect();
        assert_eq!(
            toks,
            vec![
                Token::Roll,
                Token::NumRange(1, 3),
                Token::On,
                Token::RollSpecifier("1d6".into()),
                Token::Plus,
                Token::Num(1),
                Token::Arrow,
                Token::SetFact,
                Token::Str("party is lost".into()),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn scan_roll() {
        let source = "roll 99 on 3d100".chars().collect();
        let mut scanner = Scanner::new(source);
        let toks: Vec<Token> = scanner.tokens().into_iter().map(|t| t.unwrap()).collect();
        assert_eq!(
            toks,
            vec![
                Token::Roll,
                Token::Num(99),
                Token::On,
                Token::RollSpecifier("3d100".into()),
                Token::Eof,
            ]
        )
    }

    #[test]
    fn scan_concat() {
        let source = "set-fact \"weather is \" + roll on table \"weather\""
            .chars()
            .collect();
        let mut scanner = Scanner::new(source);
        let toks: Vec<Token> = scanner.tokens().into_iter().map(|t| t.unwrap()).collect();
        assert_eq!(
            toks,
            vec![
                Token::SetFact,
                Token::Str("weather is ".into()),
                Token::Plus,
                Token::Roll,
                Token::On,
                Token::Table,
                Token::Str("weather".into()),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn scan_matching_roll() {
        let source = "roll 2d6
            \t2-4 => set-fact \"encounter is hostile\"
            \t5-8 => set-fact \"encounter is neutral\"
            end"
        .chars()
        .collect();
        let mut scanner = Scanner::new(source);
        let toks: Vec<Token> = scanner.tokens().into_iter().map(|t| t.unwrap()).collect();
        assert_eq!(
            toks,
            vec![
                Token::Roll,
                Token::RollSpecifier("2d6".into()),
                Token::Newline,
                Token::Indent,
                Token::NumRange(2, 4),
                Token::Arrow,
                Token::SetFact,
                Token::Str("encounter is hostile".into()),
                Token::Newline,
                Token::Indent,
                Token::NumRange(5, 8),
                Token::Arrow,
                Token::SetFact,
                Token::Str("encounter is neutral".into()),
                Token::Newline,
                Token::End,
                Token::Eof,
            ]
        )
    }

    #[test]
    fn scan_proc_def() {
        let source = "procedure proc\n\troll on table \"table\"\nend"
            .chars()
            .collect();
        let mut scanner = Scanner::new(source);
        let toks: Vec<Token> = scanner.tokens().into_iter().map(|t| t.unwrap()).collect();
        assert_eq!(
            toks,
            vec![
                Token::Procedure,
                Token::Identifier("proc".into()),
                Token::Newline,
                Token::Indent,
                Token::Roll,
                Token::On,
                Token::Table,
                Token::Str("table".into()),
                Token::Newline,
                Token::End,
                Token::Eof
            ]
        )
    }

    #[test]
    fn tokens_valid_once() {
        let source = "roll 2-10".chars().collect();
        let mut scanner = Scanner::new(source);
        let toks: Vec<Token> = scanner.tokens().into_iter().map(|t| t.unwrap()).collect();
        assert_eq!(toks, vec![Token::Roll, Token::NumRange(2, 10), Token::Eof]);
        assert!(scanner.tokens().is_empty());
        assert!(scanner.tokens().is_empty());
    }

    #[test]
    #[should_panic(expected = "expected '>' after '='")]
    fn incomplete_arrow() {
        let source = "= 5".chars().collect();
        let mut scanner = Scanner::new(source);
        let _ = scanner
            .tokens()
            .into_iter()
            .map(|t| t.unwrap())
            .collect::<Vec<Token>>();
    }

    #[test]
    #[should_panic(expected = "unterminated string")]
    fn unterminated_string() {
        let source = "\"Unterminated string".chars().collect();
        let mut scanner = Scanner::new(source);
        let _ = scanner
            .tokens()
            .into_iter()
            .map(|t| t.unwrap())
            .collect::<Vec<Token>>();
    }
}
