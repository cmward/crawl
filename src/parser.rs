use crate::error::CrawlError;
use crate::scanner::Token;

// TODO: replace expects with automatically filled out expected tokens in consume
// TODO: lots of cloning - Rc?
// TODO: crazy error handling

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    ClearFact(String),
    ClearPersistentFact(String),
    // NonTargetedRoll is only used for string interpolation values
    NontargetedRoll(ModifiedRollSpecifier),
    IfThen {
        antecedent: Antecedent,
        consequent: Box<Statement>,
    },
    LoadTable(String),
    MatchingRoll {
        roll_specifier: ModifiedRollSpecifier,
        arms: Vec<MatchingRollArm>,
    },
    Procedure {
        declaration: ProcedureDeclaration,
        body: Vec<Box<Statement>>,
    },
    ProcedureCall(String),
    Reminder(String),
    SetFact(CrawlStr),
    SetPersistentFact(String),
    TableRoll(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProcedureDeclaration(pub String);

#[derive(Debug, Clone, PartialEq)]
pub enum CrawlStr {
    Str(String),
    InterpolatedStr {
        format_string: String,
        expressions: Vec<Statement>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModifiedRollSpecifier {
    // These fields are public so DiceRoll can implement TryFrom<ModifiedRollSpecifier>.
    // Don't really like it, but idk what the best thing to do is.
    pub base_roll_specifier: Token,
    pub modifier: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatchingRollArm {
    pub target: Token,
    pub consequent: Statement,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Antecedent {
    CheckFact(String),
    CheckPersistentFact(String),
    DiceRoll {
        target: Token,
        roll_specifier: ModifiedRollSpecifier,
    },
}

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    position: usize, // Index of the token to be recognized
}

// TODO: `reason` in parser error

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
        let result = match self.peek() {
            Token::ClearFact => self.clear_fact(),
            Token::ClearPersistentFact => self.clear_persistent_fact(),
            Token::Identifier(_) => self.procedure_call(),
            Token::If => self.if_then(),
            Token::Load => self.load_table(),
            Token::Procedure => self.procedure(),
            Token::Reminder => self.reminder(),
            Token::Roll => match self.peek_next() {
                Token::On => self.table_roll(),
                Token::RollSpecifier(_) => self.matching_roll(),
                _ => Err(CrawlError::ParserError {
                    token: format!("{:?}", self.peek()),
                }),
            },
            Token::SetFact => dbg!(self.set_fact()),
            Token::SetPersistentFact => self.set_persistent_fact(),
            _ => Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            }),
        };

        // Try to move past errors to sync up to the next statement.
        // Should probably try something more intentional - if the result
        // is an error, advance until we can consume a newline, and try for
        // a new statement.
        if result.is_err() {
            self.advance();
        }

        self.consume(Token::Newline)?;
        while *self.peek() == Token::Newline {
            self.advance();
        }

        result
    }

    fn procedure(&mut self) -> Result<Statement, CrawlError> {
        self.consume(Token::Procedure).expect("expected procedure");

        let declaration = if let Token::Identifier(name) = self.peek() {
            Ok(ProcedureDeclaration(name.clone()))
        } else {
            Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            })
        }?;

        self.advance();
        self.consume(Token::Newline).expect("expected newline");

        let mut body = Vec::new();
        while *self.peek() != Token::End {
            self.consume(Token::Indent).expect("expected indent");
            body.push(Box::new(self.statement()?));
        }
        self.consume(Token::End).expect("expected end");

        Ok(Statement::Procedure { declaration, body })
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
            consequent: Box::new(consequent),
        })
    }

    fn matching_roll(&mut self) -> Result<Statement, CrawlError> {
        self.consume(Token::Roll).expect("expected roll");
        let roll_specifier = self.modified_specifier()?;

        self.consume(Token::Newline).expect("expected newline");

        let mut arms: Vec<MatchingRollArm> = Vec::new();
        while *self.peek() != Token::End {
            self.consume(Token::Indent).expect("expected indent");
            while *self.peek() == Token::Indent {
                self.advance();
            }

            // this is ugly - why check for end in the while loop if we never find it there?
            if *self.peek() == Token::End {
                break;
            }

            let target = match self.peek() {
                Token::Num(_) | Token::NumRange(_, _) => Ok(self.peek().clone()),
                _ => Err(CrawlError::ParserError {
                    token: format!("{:?}", self.peek()),
                }),
            }?;
            self.advance();

            self.consume(Token::Arrow).expect("expected arrow");
            let consequent = self.consequent()?;
            let arm = MatchingRollArm { target, consequent };
            arms.push(arm);

            self.consume(Token::Newline).expect("expected newline");
        }

        self.consume(Token::End).expect("expected end");

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

        let mut modifier: i32 = 0;
        match self.peek() {
            Token::Plus => {
                self.advance();
                if let Token::Num(n) = self.peek() {
                    modifier = *n;
                }
                self.advance();
            }
            Token::Minus => {
                self.advance();
                if let Token::Num(n) = self.peek() {
                    modifier = -*n;
                }
                self.advance();
            }
            _ => modifier = 0,
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

        self.advance();

        Ok(Statement::Reminder(reminder))
    }

    fn load_table(&mut self) -> Result<Statement, CrawlError> {
        self.consume(Token::Load).expect("expected load");
        self.consume(Token::Table).expect("expected table");

        let load_table = if let Token::Str(table_name) = self.peek() {
            Ok(Statement::LoadTable(table_name.clone()))
        } else {
            Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            })
        };

        self.advance();

        load_table
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

    fn consequent(&mut self) -> Result<Statement, CrawlError> {
        match self.peek() {
            Token::ClearFact => self.clear_fact(),
            Token::ClearPersistentFact => self.clear_persistent_fact(),
            Token::Identifier(_) => self.procedure_call(),
            Token::Reminder => self.reminder(),
            Token::Roll => self.table_roll(),
            Token::SetFact => self.set_fact(),
            Token::SetPersistentFact => self.set_persistent_fact(),
            _ => Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            }),
        }
    }

    fn dice_roll(&mut self) -> Result<Antecedent, CrawlError> {
        self.consume(Token::Roll).expect("expected roll");
        let target = match self.peek() {
            Token::Num(_) | Token::NumRange(_, _) => Ok(self.peek().clone()),
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

        self.advance();

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

        self.advance();

        Ok(Antecedent::CheckPersistentFact(fact?))
    }

    fn set_fact(&mut self) -> Result<Statement, CrawlError> {
        self.consume(Token::SetFact).expect("expected set-fact");
        let fact = self.str()?;

        Ok(Statement::SetFact(fact))
    }

    fn set_persistent_fact(&mut self) -> Result<Statement, CrawlError> {
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

        Ok(Statement::SetPersistentFact(fact))
    }

    fn clear_fact(&mut self) -> Result<Statement, CrawlError> {
        self.consume(Token::ClearFact).expect("expected clear-fact");
        let fact = if let Token::Str(fact) = self.peek() {
            Ok(fact.clone())
        } else {
            Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            })
        }?;

        self.advance();

        Ok(Statement::ClearFact(fact))
    }

    fn clear_persistent_fact(&mut self) -> Result<Statement, CrawlError> {
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

        Ok(Statement::ClearPersistentFact(fact))
    }

    fn table_roll(&mut self) -> Result<Statement, CrawlError> {
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

        Ok(Statement::TableRoll(table_identifier))
    }

    fn nontargeted_roll(&mut self) -> Result<Statement, CrawlError> {
        self.consume(Token::Roll).expect("expected roll");
        let spec = self.modified_specifier()?;
        Ok(Statement::NontargetedRoll(spec))
    }

    fn str(&mut self) -> Result<CrawlStr, CrawlError> {
        let s = if let Token::Str(st) = self.peek() {
            Ok(st.clone())
        } else {
            Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            })
        }?;

        self.advance();

        if let Token::Percent = *self.peek() {
            self.advance();
            let expr = match self.peek_next() {
                Token::On => self.table_roll(),
                Token::RollSpecifier(_) => self.nontargeted_roll(),
                _ => Err(CrawlError::ParserError {
                    token: format!("{:?}", self.peek()),
                }),
            }?;
            Ok(CrawlStr::InterpolatedStr {
                format_string: s,
                expressions: vec![expr],
            })
        } else {
            Ok(CrawlStr::Str(s))
        }
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn consume(&mut self, token: Token) -> Result<Token, CrawlError> {
        if *self.peek() == token {
            self.advance();
            Ok(token)
        } else {
            Err(CrawlError::ParserError {
                token: format!("{:?}", self.peek()),
            })
        }
    }

    fn peek(&self) -> &Token {
        if self.tokens.len() > self.position {
            return &self.tokens[self.position]
        }
        &Token::Eof
    }

    fn peek_next(&self) -> &Token {
        if self.tokens.len() > self.position + 1 {
            return &self.tokens[self.position + 1];
        }
        &Token::Eof
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
    fn parse_reminder() {
        let toks = vec![
            Token::Reminder,
            Token::Str("don't forget to eat".into()),
            Token::Newline,
            Token::Eof,
        ];
        let parsed = Parser::new(toks).parse();
        assert_eq!(
            parsed
                .into_iter()
                .map(|a| a.unwrap())
                .collect::<Vec<Statement>>(),
            vec![Statement::Reminder("don't forget to eat".into())],
        )
    }

    #[test]
    #[should_panic]
    fn parse_statement_no_nl() {
        let toks = vec![
            Token::Reminder,
            Token::Str("statements end with a newline".into()),
            Token::Eof,
        ];
        let _: Vec<Statement> = Parser::new(toks)
            .parse()
            .into_iter()
            .map(|a| a.unwrap())
            .collect();
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
                        modifier: 1,
                    },
                },
                consequent: Box::new(Statement::SetFact(CrawlStr::Str("cool!".into()))),
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
                    modifier: -2,
                },
                arms: vec![
                    MatchingRollArm {
                        target: Token::Num(2),
                        consequent: Statement::SetFact(CrawlStr::Str("you died".into()))
                    },
                    MatchingRollArm {
                        target: Token::NumRange(3, 40),
                        consequent: Statement::SetFact(CrawlStr::Str("you're alright".into()))
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
            Statement::SetFact(CrawlStr::Str("weather is nice".into()))
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
            Statement::SetPersistentFact("weather is nice".into())
        )
    }

    #[test]
    fn clear_fact() {
        let toks = vec![Token::ClearFact, Token::Str("weather is nice".into())];
        let parsed = Parser::new(toks).clear_fact();
        assert_eq!(
            parsed.unwrap(),
            Statement::ClearFact("weather is nice".into())
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
            Statement::ClearPersistentFact("weather is nice".into())
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
                    modifier: 5,
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
        assert_eq!(parsed.unwrap(), Statement::TableRoll("table-t1".into()))
    }
}
