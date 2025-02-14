use std::collections::HashMap;

use crate::error::CrawlError;
use crate::facts::FactDatabase;
use crate::parser::{Antecedent, Consequent, Statement};

#[derive(Debug, PartialEq)]
pub enum StatementRecord {
    Reminder(String),
    IfThen {
        antecedent: bool,
        consequent: Box<StatementRecord>,
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
            Statement::IfThen {
                antecedent,
                consequent,
            } => todo!(),
            Statement::Reminder(reminder) => self.evaluate_reminder(&statement),
            Statement::Procedure { declaration, body } => todo!(),
            Statement::ProcedureCall(procedure_name) => todo!(),
            Statement::MatchingRoll {
                roll_specifier,
                arms,
            } => todo!(),
        }
    }

    fn evaluate_if_then(
        &mut self,
        antecedent: &Antecedent,
        consequent: &Consequent,
    ) -> Result<StatementRecord, CrawlError> {
        todo!()
    }

    fn evaluate_reminder(&mut self, reminder: &Statement) -> Result<StatementRecord, CrawlError> {
        if let Statement::Reminder(reminder) = reminder {
            Ok(StatementRecord::Reminder(reminder.clone()))
        } else {
            Err(CrawlError::InterpreterError {
                reason: "expected a parsed reminder".into(),
            })
        }
    }

    fn evaluate_antecedent(
        &mut self,
        antecedent: &Antecedent,
    ) -> Result<bool, CrawlError> {
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
        consequent: &Consequent,
    ) -> Result<StatementRecord, CrawlError> {
        match consequent {
            Consequent::SetFact(fact) => todo!(),
            Consequent::ClearFact(fact) => todo!(),
            Consequent::SwapFact(fact) => todo!(),
            Consequent::SetPersistentFact(fact) => todo!(),
            Consequent::ClearPersistentFact(fact) => todo!(),
            Consequent::SwapPersistentFact(fact) => todo!(),
            Consequent::TableRoll(table_name) => todo!(),
        }
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
