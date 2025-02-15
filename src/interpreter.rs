use std::collections::HashMap;

use crate::error::CrawlError;
use crate::facts::FactDatabase;
use crate::parser::{Antecedent, Statement};

#[derive(Debug, PartialEq)]
pub enum StatementRecord {
    Reminder(String),
    IfThen {
        antecedent: bool,
        consequent: Option<Box<StatementRecord>>,
    },
}

pub struct CrawlProcedure {
    definition: Statement,
    facts: FactDatabase,
}

pub struct Interpreter {
    procedures: HashMap<String, CrawlProcedure>,
    persistent_facts: FactDatabase,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            procedures: HashMap::new(),
            persistent_facts: FactDatabase,
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
            } => todo!(),
            Statement::Procedure { declaration, body } => todo!(),
            Statement::ProcedureCall(procedure_name) => todo!(),
            Statement::Reminder(reminder) => self.evaluate_reminder(reminder),
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
            } => todo!(),
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

    fn evaluate_reminder(&mut self, reminder: &String) -> Result<StatementRecord, CrawlError> {
        Ok(StatementRecord::Reminder(reminder.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpret_reminder() {
        let ast = Statement::Reminder("players must eat rations daily".into());
        let value = Interpreter::new().evaluate_statement(&ast).unwrap();
        assert_eq!(
            value,
            StatementRecord::Reminder("players must eat rations daily".into())
        );
    }
}
