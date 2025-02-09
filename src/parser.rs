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
pub enum Antecedent {
    DiceRoll {
        target: Token,
        roll_specifier: Token,
        modifier: Option<i32>,
    },
    CheckFact(String),
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
            // TODO
            statements.push(Ok(self.statement().unwrap()));
        }
        statements
    }

    fn statement(&mut self) -> Result<Statement, CrawlError> {
        let result: Result<Statement, CrawlError>;
        match self.peek() {
            Token::Procedure => result = self.procedure(),
            Token::Identifier(_) => result = self.procedure_call(),
            Token::If => result = self.if_then(),
            _ => {
                result = Err(CrawlError::ParserError {
                    token: format!("{:?}", self.peek()),
                });
            }
        }

        self.consume(Token::Newline)?;

        result
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

    fn if_then(&mut self) -> Result<Statement, CrawlError> {
        self.consume(Token::If).expect("expected if");

        let antecedent: Antecedent;
        match self.peek() {
            Token::Roll => antecedent = self.dice_roll()?,
            Token::FactTest => antecedent = self.fact_check()?,
            _ => Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            }),
        }

        Statement::IfThen {
            antecedent,
            consequent,
        }
    }

    fn dice_roll(&mut self) -> Result<Antecedent, CrawlError> {
        self.consume(Token::Roll).expect("expected roll");

        let target: Result<Token, CrawlError>;
        match self.peek() {
            Token::NumRange(_, _) => target = Ok(self.peek().clone()),
            Token::Num(_) => target = Ok(self.peek().clone()),
            _ => {
                target = Err(CrawlError::ParserError {
                    token: format!("{:?}", self.peek()),
                })
            }
        }
        self.advance();

        self.consume(Token::On).expect("expected on");

        let roll_specifier = if let Token::RollSpecifier(_) = self.peek() {
            Ok(self.peek().clone())
        } else {
            Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            })
        };
        self.advance();

        let mut modifier: Option<i32> = None;
        match self.peek() {
            Token::Plus => {
                self.advance();
                if let Token::Num(n) = self.peek() {
                    modifier = Some(*n);
                }
            }
            Token::Minus => {
                self.advance();
                if let Token::Num(n) = self.peek() {
                    modifier = Some(-*n);
                }
            }
            _ => modifier = None,
        }

        Ok(Antecedent::DiceRoll {
            target: target?,
            roll_specifier: roll_specifier?,
            modifier,
        })
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
