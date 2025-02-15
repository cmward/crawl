use std::collections::HashMap;

use crate::dice::DiceRoll;
use crate::error::CrawlError;
use crate::facts::FactDatabase;
use crate::parser::{
    Antecedent, MatchingRollArm, ModifiedRollSpecifier, ProcedureDeclaration, Statement,
};
use crate::scanner::Token;

#[derive(Debug, PartialEq)]
pub enum StatementRecord {
    IfThen {
        antecedent: bool,
        consequent: Option<Box<StatementRecord>>,
    },
    MatchingRoll {
        matched_target: Option<Token>,
        consequent: Option<Box<StatementRecord>>,
    },
    ProcedureCall {
        records: Vec<Box<StatementRecord>>,
    },
    ProcedureDefinition(String),
    Reminder(String),
}

#[derive(Debug)]
pub struct CrawlProcedure {
    identifier: String,
    body: Vec<Statement>,
    facts: FactDatabase,
}

impl CrawlProcedure {
    pub fn new(identifier: String, body: Vec<Statement>, facts: FactDatabase) -> Self {
        CrawlProcedure {
            identifier,
            body,
            facts,
        }
    }
}

pub struct Interpreter<'a> {
    procedures: HashMap<String, CrawlProcedure>,
    persistent_facts: FactDatabase,
    facts_stack: Vec<&'a FactDatabase>,
}

impl<'a> Default for Interpreter<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Interpreter<'a> {
    pub fn new() -> Self {
        Interpreter {
            procedures: HashMap::new(),
            persistent_facts: FactDatabase,
            facts_stack: Vec::new(),
        }
    }

    pub fn interpret(
        &mut self,
        statements: Vec<Statement>,
    ) -> Vec<Result<StatementRecord, CrawlError>> {
        let mut records = Vec::new();
        for statement in statements {
            records.push(self.evaluate_statement(&statement));
        }
        records
    }

    fn evaluate_statement(&mut self, statement: &Statement) -> Result<StatementRecord, CrawlError> {
        match statement {
            Statement::ClearFact(fact) => todo!(),
            Statement::ClearPersistentFact(fact) => todo!(),
            Statement::IfThen {
                antecedent,
                consequent,
            } => self.evaluate_if_then(antecedent, consequent),
            Statement::MatchingRoll {
                roll_specifier,
                arms,
            } => self.evaluate_matching_roll(roll_specifier, arms),
            Statement::Procedure { declaration, body } => {
                // TODO: How to avoid the vec copy?
                self.evaluate_procedure_definition(declaration, body.to_vec())
            }
            Statement::ProcedureCall(procedure_name) => {
                self.evaluate_procedure_call(procedure_name)
            }
            Statement::Reminder(reminder) => self.evaluate_reminder(reminder),
            // Can you {operation}_fact as a top-level statement? What would that mean/do? How to detect?
            Statement::SetFact(fact) => todo!(),
            Statement::SetPersistentFact(fact) => todo!(),
            Statement::SwapFact(fact) => todo!(),
            Statement::SwapPersistentFact(fact) => todo!(),
            Statement::TableRoll(table_name) => todo!(),
        }
    }

    fn evaluate_antecedent(&mut self, antecedent: &Antecedent) -> Result<bool, CrawlError> {
        match antecedent {
            Antecedent::CheckFact(fact) => todo!(),
            Antecedent::CheckPersistentFact(fact) => todo!(),
            Antecedent::DiceRoll {
                target,
                roll_specifier,
            } => self.evaluate_dice_roll(target, roll_specifier),
        }
    }

    fn evaluate_consequent(
        &mut self,
        consequent: &Statement,
    ) -> Result<StatementRecord, CrawlError> {
        match consequent {
            Statement::ClearFact(fact) => todo!(),
            Statement::ClearPersistentFact(fact) => todo!(),
            Statement::SetFact(fact) => todo!(),
            Statement::SetPersistentFact(fact) => todo!(),
            Statement::Reminder(reminder) => self.evaluate_reminder(reminder),
            Statement::SwapFact(fact) => todo!(),
            Statement::SwapPersistentFact(fact) => todo!(),
            Statement::TableRoll(table_name) => todo!(),
            _ => Err(CrawlError::InterpreterError {
                reason: "Invalid statement as consequent".into(),
            }),
        }
    }

    fn evaluate_if_then(
        &mut self,
        antecedent: &Antecedent,
        consequent: &Statement,
    ) -> Result<StatementRecord, CrawlError> {
        let antecedent_value = self.evaluate_antecedent(antecedent)?;
        if antecedent_value {
            Ok(StatementRecord::IfThen {
                antecedent: antecedent_value,
                consequent: Some(Box::new(self.evaluate_consequent(consequent)?)),
            })
        } else {
            Ok(StatementRecord::IfThen {
                antecedent: antecedent_value,
                consequent: None,
            })
        }
    }

    fn evaluate_reminder(&mut self, reminder: &str) -> Result<StatementRecord, CrawlError> {
        Ok(StatementRecord::Reminder(reminder.to_string()))
    }

    fn evaluate_matching_roll(
        &mut self,
        modified_roll_specifier: &ModifiedRollSpecifier,
        arms: &[MatchingRollArm],
    ) -> Result<StatementRecord, CrawlError> {
        for arm in arms {
            if self.evaluate_dice_roll(&arm.target, modified_roll_specifier)? {
                return Ok(StatementRecord::MatchingRoll {
                    matched_target: Some(arm.target.clone()),
                    consequent: Some(Box::new(self.evaluate_consequent(&arm.consequent)?)),
                });
            }
        }

        Ok(StatementRecord::MatchingRoll {
            matched_target: None,
            consequent: None,
        })
    }

    fn evaluate_procedure_definition(
        &mut self,
        declaration: &ProcedureDeclaration,
        body: Vec<Box<Statement>>,
    ) -> Result<StatementRecord, CrawlError> {
        let ident = declaration.0.clone();
        let def = CrawlProcedure::new(
            ident.clone(),
            body.into_iter().map(|s| *s).collect(),
            FactDatabase::new(),
        );
        self.procedures.insert(ident.clone(), def);
        Ok(StatementRecord::ProcedureDefinition(ident.clone()))
    }

    fn evaluate_procedure_call(
        &mut self,
        procedure_identifier: &str,
    ) -> Result<StatementRecord, CrawlError> {
        let proc = self.procedures.get(procedure_identifier);
        todo!()
    }

    fn evaluate_dice_roll(
        &mut self,
        target: &Token,
        modified_roll_specifier: &ModifiedRollSpecifier,
    ) -> Result<bool, CrawlError> {
        let roll: DiceRoll = modified_roll_specifier.try_into()?;
        let roll_result = roll.roll();
        match target {
            Token::Num(n) => Ok(roll_result.total == *n),
            Token::NumRange(min, max) => Ok(*min <= roll_result.total && roll_result.total <= *max),
            _ => Err(CrawlError::InterpreterError {
                reason: "invalid roll target".into(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpret_reminder() {
        let ast = Statement::Reminder("players must eat rations daily".into());
        let value: Vec<StatementRecord> = Interpreter::new()
            .interpret(vec![ast])
            .into_iter()
            .map(|v| v.unwrap())
            .collect();
        assert_eq!(
            value,
            vec![StatementRecord::Reminder(
                "players must eat rations daily".into()
            )],
        );
    }

    #[test]
    fn interpret_if_then_antecedent_true() {
        let ast = Statement::IfThen {
            antecedent: Antecedent::DiceRoll {
                target: Token::Num(1),
                roll_specifier: ModifiedRollSpecifier {
                    base_roll_specifier: Token::RollSpecifier("1d1".into()),
                    modifier: 0,
                },
            },
            consequent: Box::new(Statement::Reminder("you passed the check".into())),
        };
        let value: Vec<StatementRecord> = Interpreter::new()
            .interpret(vec![ast])
            .into_iter()
            .map(|v| v.unwrap())
            .collect();
        assert_eq!(
            value,
            vec![StatementRecord::IfThen {
                antecedent: true,
                consequent: Some(Box::new(StatementRecord::Reminder(
                    "you passed the check".into()
                )))
            }]
        );
    }

    #[test]
    fn interpret_if_then_antecedent_false() {
        let ast = Statement::IfThen {
            antecedent: Antecedent::DiceRoll {
                target: Token::Num(100),
                roll_specifier: ModifiedRollSpecifier {
                    base_roll_specifier: Token::RollSpecifier("1d1".into()),
                    modifier: 0,
                },
            },
            consequent: Box::new(Statement::Reminder("you passed the check".into())),
        };
        let value: Vec<StatementRecord> = Interpreter::new()
            .interpret(vec![ast])
            .into_iter()
            .map(|v| v.unwrap())
            .collect();
        assert_eq!(
            value,
            vec![StatementRecord::IfThen {
                antecedent: false,
                consequent: None,
            }]
        );
    }

    #[test]
    fn interpret_proc_def() {
        let body = vec![
            Box::new(Statement::IfThen {
                antecedent: Antecedent::DiceRoll {
                    target: Token::Num(1),
                    roll_specifier: ModifiedRollSpecifier {
                        base_roll_specifier: Token::RollSpecifier("1d1".into()),
                        modifier: 0,
                    },
                },
                consequent: Box::new(Statement::Reminder("you passed the check".into())),
            }),
            Box::new(Statement::Reminder("cool procedure".into())),
        ];
        let ast = Statement::Procedure {
            declaration: ProcedureDeclaration("proc-name".into()),
            body: body.clone(),
        };
        let mut interp = Interpreter::new();
        let value: Vec<StatementRecord> = interp
            .interpret(vec![ast])
            .into_iter()
            .map(|v| v.unwrap())
            .collect();
        assert_eq!(
            value,
            vec![StatementRecord::ProcedureDefinition("proc-name".into())]
        );
        assert!(interp.procedures.contains_key("proc-name"));
        let deref_body: Vec<Statement> = body.into_iter().map(|s| *s).collect();
        assert_eq!(
            *interp.procedures.get("proc-name").unwrap().body,
            dbg!(deref_body),
        );
    }

    #[test]
    fn interpret_matching_roll() {
        let ast = Statement::MatchingRoll {
            roll_specifier: ModifiedRollSpecifier {
                base_roll_specifier: Token::RollSpecifier("1d1".into()),
                modifier: 0,
            },
            arms: vec![MatchingRollArm {
                target: Token::Num(1),
                consequent: Statement::Reminder("matched 1".into()),
            }],
        };
        let value: Vec<StatementRecord> = Interpreter::new()
            .interpret(vec![ast])
            .into_iter()
            .map(|v| v.unwrap())
            .collect();
        assert_eq!(
            value,
            vec![StatementRecord::MatchingRoll {
                matched_target: Some(Token::Num(1)),
                consequent: Some(Box::new(StatementRecord::Reminder("matched 1".into()))),
            }]
        )
    }

    #[test]
    fn reminder() {
        let ast = Statement::Reminder("players must eat rations daily".into());
        let value = Interpreter::new().evaluate_statement(&ast).unwrap();
        assert_eq!(
            value,
            StatementRecord::Reminder("players must eat rations daily".into())
        );
    }
}
