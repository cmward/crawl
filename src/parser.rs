use crate::error::CrawlError;
use crate::scanner::Token;

#[derive(Debug, PartialEq)]
pub enum Statement {
    Procedure {
        declaration: ProcedureDeclaration,
        body: Vec<Box<Statement>>,
    },
    ProcedureCall(String),
    IfThen {
        antecedent: Box<Statement>,
        consequent: Consequent,
    },
    MatchingRoll,
    Reminder(String),
}

#[derive(Debug, PartialEq)]
pub struct ProcedureDeclaration(String);

#[derive(Debug, PartialEq)]
pub enum Consequent {
    SetFact(String),
    SetPFact(String),
    ClearFact(String),
    ClearPFact(String),
    TableRoll(String),
}

#[derive(Debug, PartialEq)]
pub struct DiceRoll {
    target: Token,
    roll_specifier: Token,
    modifier: i32,
}

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    position: usize, // Index of the token to be recognized
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            position: 0,
        }
    }

    pub fn parse(&mut self) -> Vec<Result<Statement, CrawlError>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(Ok(self.statement().unwrap()));
        }
        statements
    }

    fn statement(&mut self) -> Result<Statement, CrawlError> {
        let stmt: Result<Statement, CrawlError>;
        dbg!(self.peek());
        match self.peek() {
            Token::Procedure => stmt = self.procedure(),
            Token::Identifier(_) => stmt = self.procedure_call(),
            // TODO: no catchall
            _ => {
                stmt = Err(CrawlError::ParserError {
                    token: format!("{:?}", self.peek()),
                });
            }
        }
        self.consume(Token::Newline)?;
        stmt
    }

    fn procedure(&mut self) -> Result<Statement, CrawlError> {
        // TODO: replace expects with automatically filled out expected tokens in consume
        self.consume(Token::Procedure).expect("expected procedure");
        let declaration: ProcedureDeclaration;
        if let Token::Identifier(name) = self.peek() {
            declaration = ProcedureDeclaration(name.clone());
            self.advance();
            self.consume(Token::Newline).expect("expected newline");
        } else {
            return Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            });
        }

        let mut body = Vec::new();
        while *self.peek() != Token::End {
            self.consume(Token::Indent).expect("expected indent");
            body.push(Box::new(self.statement()?));
        }
        self.consume(Token::End).expect("expected end");

        return Ok(Statement::Procedure { declaration, body });
    }

    fn procedure_call(&mut self) -> Result<Statement, CrawlError> {
        if let Token::Identifier(name) = self.peek().clone() {
            self.advance();
            Ok(Statement::ProcedureCall(name.to_string()))
        } else {
            Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            })
        }
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn consume(&mut self, token: Token) -> Result<Token, CrawlError> {
        if token == *self.peek() {
            self.advance();
            Ok(token)
        } else {
            Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            })
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn is_at_end(&self) -> bool {
        *self.peek() == Token::Eof
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_procedure_call() {
        let toks = vec![
            Token::Identifier("proc-name".into()),
            Token::Newline,
            Token::Eof,
        ];
        let parsed: Vec<Statement> = Parser::new(toks)
            .parse()
            .into_iter()
            .map(|t| t.unwrap())
            .collect();
        assert_eq!(parsed, vec![Statement::ProcedureCall("proc-name".into())]);
    }

    #[test]
    fn test_parse_procedure_def() {
        let toks = vec![
            Token::Procedure,
            Token::Identifier("proc".into()),
            Token::Newline,
            Token::Indent,
            Token::Identifier("other-proc".into()),
            Token::Newline,
            Token::End,
            Token::Newline,
            Token::Eof,
        ];
        let parsed: Vec<Statement> = Parser::new(toks)
            .parse()
            .into_iter()
            .map(|t| t.unwrap())
            .collect();
        assert_eq!(
            parsed,
            vec![Statement::Procedure {
                declaration: ProcedureDeclaration("proc".into()),
                body: vec![Box::new(Statement::ProcedureCall("other-proc".into()))]
            }]
        );
    }
}
