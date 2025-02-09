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
        antecedent: Antecedent,
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
    SetPersistentFact(String),
    ClearFact(String),
    ClearPersistentFact(String),
    SwapFact(String),
    SwapPersistentFact(String),
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
    CheckPersistentFact(String),
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

        let antecedent = self.antecedent()?;
        let consequent = self.consequent()?;

        Ok(Statement::IfThen {
            antecedent,
            consequent,
        })
    }

    fn antecedent(&mut self) -> Result<Antecedent, CrawlError> {
        match self.peek() {
            Token::Roll => self.dice_roll(),
            Token::FactTest => self.fact_check(),
            Token::PersistentFactTest => self.persistent_fact_check(),
            _ => Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            }),
        }
    }

    fn consequent(&mut self) -> Result<Consequent, CrawlError> {
        match self.peek() {
            Token::SetFact => self.set_fact(),
            Token::SetPersistentFact => self.set_persistent_fact(),
            Token::ClearFact => self.clear_fact(),
            Token::ClearPersistentFact => self.clear_persistent_fact(),
            Token::SwapFact => self.swap_fact(),
            Token::SwapPersistentFact => self.swap_persistent_fact(),
            Token::Roll => self.table_roll(),
            _ => todo!(),
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

    fn fact_check(&mut self) -> Result<Antecedent, CrawlError> {
        self.consume(Token::FactTest).expect("expected fact?");
        let fact = if let Token::Str(fact) = self.peek() {
            Ok(fact.clone())
        } else {
            Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            })
        };
        Ok(Antecedent::CheckFact(fact?))
    }

    fn persistent_fact_check(&mut self) -> Result<Antecedent, CrawlError> {
        self.consume(Token::PersistentFactTest)
            .expect("expected persistent-fact?");
        let fact = if let Token::Str(fact) = self.peek() {
            Ok(fact.clone())
        } else {
            Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            })
        };
        Ok(Antecedent::CheckPersistentFact(fact?))
    }

    fn set_fact(&mut self) -> Result<Consequent, CrawlError> {
        self.consume(Token::SetFact).expect("expected set-fact");
        let fact = if let Token::Str(fact) = self.peek() {
            Ok(fact.clone())
        } else {
            Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            })
        };
        Ok(Consequent::SetFact(fact?))
    }

    fn set_persistent_fact(&mut self) -> Result<Consequent, CrawlError> {
        self.consume(Token::SetFact)
            .expect("expected set-persistent-fact");
        let fact = if let Token::Str(fact) = self.peek() {
            Ok(fact.clone())
        } else {
            Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            })
        };
        Ok(Consequent::SetPersistentFact(fact?))
    }

    fn clear_fact(&mut self) -> Result<Consequent, CrawlError> {
        self.consume(Token::ClearFact).expect("expected clear-fact");
        let fact = if let Token::Str(fact) = self.peek() {
            Ok(fact.clone())
        } else {
            Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            })
        };
        Ok(Consequent::ClearFact(fact?))
    }

    fn clear_persistent_fact(&mut self) -> Result<Consequent, CrawlError> {
        self.consume(Token::ClearPersistentFact)
            .expect("expected clear-persistent-fact");
        let fact = if let Token::Str(fact) = self.peek() {
            Ok(fact.clone())
        } else {
            Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            })
        };
        Ok(Consequent::ClearPersistentFact(fact?))
    }

    fn swap_fact(&mut self) -> Result<Consequent, CrawlError> {
        self.consume(Token::SwapFact).expect("expected swap-fact");
        let fact = if let Token::Str(fact) = self.peek() {
            Ok(fact.clone())
        } else {
            Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            })
        };
        Ok(Consequent::SwapFact(fact?))
    }

    fn swap_persistent_fact(&mut self) -> Result<Consequent, CrawlError> {
        self.consume(Token::SwapPersistentFact)
            .expect("expected swap-persistent-fact");
        let fact = if let Token::Str(fact) = self.peek() {
            Ok(fact.clone())
        } else {
            Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            })
        };
        Ok(Consequent::SwapPersistentFact(fact?))
    }

    fn table_roll(&mut self) -> Result<Consequent, CrawlError> {
        self.consume(Token::Roll).expect("expected roll");
        self.consume(Token::On).expect("expected on");
        self.consume(Token::Table).expect("expected table");
        let table_identifier = if let Token::Str(table_id) = self.peek() {
            Ok(table_id.to_string())
        } else {
            Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            })
        };
        Ok(Consequent::TableRoll(table_identifier?))
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
