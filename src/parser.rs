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
            statements.push(self.statement());
        }
        statements
    }

    fn statement(&mut self) -> Result<Statement, CrawlError> {
        match self.peek() {
            Token::Procedure => self.procedure(),
            Token::Identifier(_) => self.procedure_call(),
            _ => Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            }),
        }
    }

    fn procedure(&mut self) -> Result<Statement, CrawlError> {
        self.consume(Token::Procedure)?;
        let declaration: ProcedureDeclaration;
        if let Token::Identifier(name) = self.peek() {
            declaration = ProcedureDeclaration(name.clone());
        } else {
            return Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            });
        }

        let mut body = Vec::new();
        while *self.peek() != Token::End {
            body.push(Box::new(self.statement()?));
        }

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
        let toks = vec![Token::Identifier("proc-name".into()), Token::Eof];
        let parsed: Vec<Statement> = Parser::new(toks)
            .parse()
            .into_iter()
            .map(|t| t.unwrap())
            .collect();
        assert_eq!(parsed, vec![Statement::ProcedureCall("proc-name".into())]);
    }

    #[test]
    fn test_parse_procedure() {
        let toks = vec![
            Token::Procedure,
            Token::Identifier("proc".into()),
            Token::Newline,
            Token::Indent,
            Token::Identifier("other-proc".into()),
            Token::Newline,
            Token::End,
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
