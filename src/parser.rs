use crate::error::CrawlError;
use crate::scanner::Token;

// TODO: lots of logic duplication, e.g., all the fact and dice roll related parsing
// TODO: replace expects with automatically filled out expected tokens in consume
// TODO: lots of cloning

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
    MatchingRoll {
        roll_specifier: ModifiedRollSpecifier,
        arms: Vec<MatchingRollArm>,
    },
    Reminder(String),
}

#[derive(Debug, PartialEq)]
pub struct ProcedureDeclaration(String);

#[derive(Debug, PartialEq)]
pub struct ModifiedRollSpecifier {
    base_roll_specifier: Token,
    modifier: Option<i32>,
}

#[derive(Debug, PartialEq)]
pub struct MatchingRollArm {
    target: Token,
    consequent: Consequent,
}

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
        roll_specifier: ModifiedRollSpecifier,
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
        let result = match self.peek() {
            Token::Procedure => self.procedure(),
            Token::Identifier(_) => self.procedure_call(),
            Token::If => self.if_then(),
            Token::Roll => self.matching_roll(),
            Token::Reminder => self.reminder(),
            _ => Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            }),
        };

        self.consume(Token::Newline)?;

        result
    }

    fn procedure(&mut self) -> Result<Statement, CrawlError> {
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
        self.consume(Token::Arrow).expect("expected arrow");
        let consequent = self.consequent()?;

        Ok(Statement::IfThen {
            antecedent,
            consequent,
        })
    }

    fn matching_roll(&mut self) -> Result<Statement, CrawlError> {
        self.consume(Token::Roll).expect("expected roll");
        let roll_specifier = self.modified_specifier()?;

        self.consume(Token::Newline).expect("expected newline");

        let mut arms: Vec<MatchingRollArm> = Vec::new();
        while *self.peek() != Token::End {
            self.consume(Token::Indent).expect("expected indent");

            let target = match self.peek() {
                Token::NumRange(_, _) => Ok(self.peek().clone()),
                Token::Num(_) => Ok(self.peek().clone()),
                _ => Err(CrawlError::ParserError {
                    token: format!("{:?}", self.peek()),
                }),
            }?;
            self.advance();

            self.consume(Token::Arrow).expect("expected arrow");
            let consequent = self.consequent()?;
            dbg!(&consequent);
            let arm = MatchingRollArm { target, consequent };
            arms.push(arm);

            self.consume(Token::Newline).expect("expected newline");
        }

        Ok(Statement::MatchingRoll {
            roll_specifier,
            arms,
        })
    }

    fn modified_specifier(&mut self) -> Result<ModifiedRollSpecifier, CrawlError> {
        let base_roll_specifier = if let Token::RollSpecifier(_) = self.peek() {
            Ok(self.peek().clone())
        } else {
            Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            })
        }?;
        self.advance();

        let mut modifier: Option<i32> = None;
        match self.peek() {
            Token::Plus => {
                self.advance();
                if let Token::Num(n) = self.peek() {
                    modifier = Some(*n);
                }
                self.advance();
            }
            Token::Minus => {
                self.advance();
                if let Token::Num(n) = self.peek() {
                    modifier = Some(-*n);
                }
                self.advance();
            }
            _ => modifier = None,
        }

        Ok(ModifiedRollSpecifier {
            base_roll_specifier,
            modifier,
        })
    }

    fn reminder(&mut self) -> Result<Statement, CrawlError> {
        self.consume(Token::Reminder).expect("expected reminder");

        let reminder = if let Token::Str(reminder) = self.peek() {
            Ok(reminder.clone())
        } else {
            Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            })
        }?;

        Ok(Statement::Reminder(reminder))
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
            _ => Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            }),
        }
    }

    fn dice_roll(&mut self) -> Result<Antecedent, CrawlError> {
        self.consume(Token::Roll).expect("expected roll");

        let target = match self.peek() {
            Token::NumRange(_, _) => Ok(self.peek().clone()),
            Token::Num(_) => Ok(self.peek().clone()),
            _ => Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            }),
        }?;
        self.advance();

        self.consume(Token::On).expect("expected on");

        let roll_specifier = self.modified_specifier()?;

        Ok(Antecedent::DiceRoll {
            target,
            roll_specifier,
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
        }?;
        self.advance();
        Ok(Consequent::SetFact(fact))
    }

    fn set_persistent_fact(&mut self) -> Result<Consequent, CrawlError> {
        self.consume(Token::SetPersistentFact)
            .expect("expected set-persistent-fact");
        let fact = if let Token::Str(fact) = self.peek() {
            Ok(fact.clone())
        } else {
            Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            })
        }?;
        self.advance();
        Ok(Consequent::SetPersistentFact(fact))
    }

    fn clear_fact(&mut self) -> Result<Consequent, CrawlError> {
        self.consume(Token::ClearFact).expect("expected clear-fact");
        let fact = if let Token::Str(fact) = self.peek() {
            Ok(fact.clone())
        } else {
            Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            })
        }?;
        self.advance();
        Ok(Consequent::ClearFact(fact))
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
        }?;
        self.advance();
        Ok(Consequent::ClearPersistentFact(fact))
    }

    fn swap_fact(&mut self) -> Result<Consequent, CrawlError> {
        self.consume(Token::SwapFact).expect("expected swap-fact");
        let fact = if let Token::Str(fact) = self.peek() {
            Ok(fact.clone())
        } else {
            Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            })
        }?;
        self.advance();
        Ok(Consequent::SwapFact(fact))
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
        }?;
        self.advance();
        Ok(Consequent::SwapPersistentFact(fact))
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
        }?;
        self.advance();
        Ok(Consequent::TableRoll(table_identifier))
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
    fn parse_procedure_call() {
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
    fn parse_procedure_def() {
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

    #[test]
    fn if_then() {
        let toks = vec![
            Token::If,
            Token::Roll,
            Token::Num(6),
            Token::On,
            Token::RollSpecifier("1d6".into()),
            Token::Plus,
            Token::Num(1),
            Token::Arrow,
            Token::SetFact,
            Token::Str("cool!".into()),
        ];
        let parsed = Parser::new(toks).if_then();
        assert_eq!(
            parsed.unwrap(),
            Statement::IfThen {
                antecedent: Antecedent::DiceRoll {
                    target: Token::Num(6),
                    roll_specifier: ModifiedRollSpecifier {
                        base_roll_specifier: Token::RollSpecifier("1d6".into()),
                        modifier: Some(1),
                    },
                },
                consequent: Consequent::SetFact("cool!".into())
            }
        )
    }

    #[test]
    fn matching_roll() {
        let toks = vec![
            Token::Roll,
            Token::RollSpecifier("2d20".into()),
            Token::Minus,
            Token::Num(2),
            Token::Newline,
            Token::Indent,
            Token::Num(2),
            Token::Arrow,
            Token::SetFact,
            Token::Str("you died".into()),
            Token::Newline,
            Token::Indent,
            Token::NumRange(3, 40),
            Token::Arrow,
            Token::SetFact,
            Token::Str("you're alright".into()),
            Token::Newline,
            Token::End,
        ];
        let parsed = Parser::new(toks).matching_roll();
        assert_eq!(
            parsed.unwrap(),
            Statement::MatchingRoll {
                roll_specifier: ModifiedRollSpecifier {
                    base_roll_specifier: Token::RollSpecifier("2d20".into()),
                    modifier: Some(-2),
                },
                arms: vec![
                    MatchingRollArm {
                        target: Token::Num(2),
                        consequent: Consequent::SetFact("you died".into())
                    },
                    MatchingRollArm {
                        target: Token::NumRange(3, 40),
                        consequent: Consequent::SetFact("you're alright".into())
                    },
                ]
            }
        )
    }

    #[test]
    fn set_fact() {
        let toks = vec![Token::SetFact, Token::Str("weather is nice".into())];
        let parsed = Parser::new(toks).set_fact();
        assert_eq!(
            parsed.unwrap(),
            Consequent::SetFact("weather is nice".into())
        )
    }

    #[test]
    fn set_pfact() {
        let toks = vec![
            Token::SetPersistentFact,
            Token::Str("weather is nice".into()),
        ];
        let parsed = Parser::new(toks).set_persistent_fact();
        assert_eq!(
            parsed.unwrap(),
            Consequent::SetPersistentFact("weather is nice".into())
        )
    }

    #[test]
    fn clear_fact() {
        let toks = vec![Token::ClearFact, Token::Str("weather is nice".into())];
        let parsed = Parser::new(toks).clear_fact();
        assert_eq!(
            parsed.unwrap(),
            Consequent::ClearFact("weather is nice".into())
        )
    }

    #[test]
    fn clear_pfact() {
        let toks = vec![
            Token::ClearPersistentFact,
            Token::Str("weather is nice".into()),
        ];
        let parsed = Parser::new(toks).clear_persistent_fact();
        assert_eq!(
            parsed.unwrap(),
            Consequent::ClearPersistentFact("weather is nice".into())
        )
    }

    #[test]
    fn swap_fact() {
        let toks = vec![Token::SwapFact, Token::Str("weather is nice".into())];
        let parsed = Parser::new(toks).swap_fact();
        assert_eq!(
            parsed.unwrap(),
            Consequent::SwapFact("weather is nice".into())
        )
    }

    #[test]
    fn swap_persistent_fact() {
        let toks = vec![
            Token::SwapPersistentFact,
            Token::Str("weather is nice".into()),
        ];
        let parsed = Parser::new(toks).swap_persistent_fact();
        assert_eq!(
            parsed.unwrap(),
            Consequent::SwapPersistentFact("weather is nice".into())
        )
    }

    #[test]
    fn reminder() {
        let toks = vec![Token::Reminder, Token::Str("don't forget to eat".into())];
        let parsed = Parser::new(toks).reminder();
        assert_eq!(
            parsed.unwrap(),
            Statement::Reminder("don't forget to eat".into())
        )
    }

    #[test]
    fn dice_roll() {
        let toks = vec![
            Token::Roll,
            Token::NumRange(1, 5),
            Token::On,
            Token::RollSpecifier("1d12".into()),
            Token::Plus,
            Token::Num(5),
        ];
        let parsed = Parser::new(toks).dice_roll();
        assert_eq!(
            parsed.unwrap(),
            Antecedent::DiceRoll {
                target: Token::NumRange(1, 5),
                roll_specifier: ModifiedRollSpecifier {
                    base_roll_specifier: Token::RollSpecifier("1d12".into()),
                    modifier: Some(5),
                }
            }
        )
    }

    #[test]
    fn table_roll() {
        let toks = vec![
            Token::Roll,
            Token::On,
            Token::Table,
            Token::Str("table-t1".into()),
        ];
        let parsed = Parser::new(toks).table_roll();
        assert_eq!(parsed.unwrap(), Consequent::TableRoll("table-t1".into()))
    }
}
