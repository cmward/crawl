use regex::Regex;
use std::borrow::Cow;
use std::collections::HashMap;

use crate::dice::{DiceRoll, DiceRollResult};
use crate::error::CrawlError;
use crate::facts::FactDatabase;
use crate::parser::{
    Antecedent, CrawlStr, MatchingRollArm, ModifiedRollSpecifier, ProcedureDeclaration, Statement,
};
use crate::scanner::Token;
use crate::tables::Table;

#[derive(Debug, PartialEq)]
pub enum StatementRecord {
    CheckFact(bool),
    CheckPersistentFact(bool),
    ClearFact(String),
    ClearPersistentFact(String),
    IfThen {
        antecedent: bool,
        consequent: Option<Box<StatementRecord>>,
    },
    LoadTable(String),
    MatchingRoll {
        matched_target: Option<Token>,
        consequent: Option<Box<StatementRecord>>,
    },
    NontargetedRoll(i32),
    ProcedureCall {
        identifier: String,
        records: Vec<Box<StatementRecord>>,
    },
    ProcedureDefinition(String),
    Reminder(String),
    SetFact(String),
    SetPersistentFact(String),
    TableRoll(String),
}

#[derive(Debug)]
pub struct CrawlProcedure {
    identifier: String,
    body: Vec<Statement>,
}

impl CrawlProcedure {
    pub fn new(identifier: String, body: Vec<Statement>) -> Self {
        CrawlProcedure { identifier, body }
    }
}

pub struct Interpreter {
    procedures: HashMap<String, CrawlProcedure>,
    tables: HashMap<String, Table>,
    pub persistent_facts: FactDatabase,
    pub local_facts: FactDatabase,
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
            tables: HashMap::new(),
            persistent_facts: FactDatabase::default(),
            local_facts: FactDatabase::default(),
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
            Statement::ClearFact(fact) => self.evaluate_clear_fact(fact.clone()),
            Statement::ClearPersistentFact(fact) => {
                self.evaluate_clear_persistent_fact(fact.clone())
            }
            Statement::IfThen {
                antecedent,
                consequent,
            } => self.evaluate_if_then(antecedent, consequent),
            Statement::LoadTable(table_name) => self.evaluate_load_table(table_name.clone()),
            Statement::MatchingRoll {
                roll_specifier,
                arms,
            } => self.evaluate_matching_roll(roll_specifier, arms),
            Statement::Procedure { declaration, body } => {
                // How to avoid the vec copy?
                self.evaluate_procedure_definition(
                    declaration,
                    body.iter().cloned().map(|s| *s).collect(),
                )
            }
            Statement::ProcedureCall(identifier) => self.evaluate_procedure_call(identifier),
            Statement::Reminder(reminder) => self.evaluate_reminder(reminder.clone()),
            // Can you {operation}_fact as a top-level statement? What would that mean/do?
            Statement::SetFact(fact) => self.evaluate_set_fact(fact.clone()),
            Statement::SetPersistentFact(fact) => self.evaluate_set_persistent_fact(fact.clone()),
            Statement::TableRoll(table_name) => self.evaluate_table_roll(table_name),
            Statement::NontargetedRoll(specifier) => self.evaluate_nontargeted_roll(specifier),
        }
    }

    fn evaluate_antecedent(&mut self, antecedent: &Antecedent) -> Result<bool, CrawlError> {
        match antecedent {
            Antecedent::CheckFact(fact) => self.evaluate_check_fact(fact.clone()),
            Antecedent::CheckPersistentFact(fact) => {
                self.evaluate_check_persistent_fact(fact.clone())
            }
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
            Statement::ClearFact(fact) => self.evaluate_clear_fact(fact.clone()),
            Statement::ClearPersistentFact(fact) => {
                self.evaluate_clear_persistent_fact(fact.clone())
            }
            Statement::ProcedureCall(procedure_identifier) => {
                self.evaluate_procedure_call(procedure_identifier)
            }
            Statement::SetFact(fact) => self.evaluate_set_fact(fact.clone()),
            Statement::SetPersistentFact(fact) => self.evaluate_set_persistent_fact(fact.clone()),
            Statement::Reminder(reminder) => self.evaluate_reminder(reminder.clone()),
            Statement::TableRoll(table_name) => self.evaluate_table_roll(table_name),
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

    fn evaluate_reminder(&self, reminder: String) -> Result<StatementRecord, CrawlError> {
        Ok(StatementRecord::Reminder(reminder))
    }

    fn evaluate_load_table(&mut self, table_name: String) -> Result<StatementRecord, CrawlError> {
        let table_load = Table::load(&table_name);
        match table_load {
            Ok(table) => {
                self.tables.insert(table_name.clone(), table);
                Ok(StatementRecord::LoadTable(table_name))
            }
            Err(error) => Err(CrawlError::InterpreterError {
                reason: format!("Failed to load table {table_name} ({error})"),
            }),
        }
    }

    fn evaluate_table_roll(&mut self, table_name: &str) -> Result<StatementRecord, CrawlError> {
        let table = self.tables.get(table_name).unwrap();
        // TODO: support `roll 1d6 + 3 on table "crits"`
        let table_roll_result = table.auto_roll()?;
        Ok(StatementRecord::TableRoll(
            table_roll_result.entry.value.clone(),
        ))
    }

    fn evaluate_matching_roll(
        &mut self,
        modified_roll_specifier: &ModifiedRollSpecifier,
        arms: &[MatchingRollArm],
    ) -> Result<StatementRecord, CrawlError> {
        let roll: DiceRoll = modified_roll_specifier.try_into()?;
        let roll_result = roll.roll();
        for arm in arms {
            if self.roll_result_matches_target(&roll_result, &arm.target)? {
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
        body: Vec<Statement>,
    ) -> Result<StatementRecord, CrawlError> {
        let ident = declaration.0.clone();
        let def = CrawlProcedure::new(ident.clone(), body);
        self.procedures.insert(def.identifier.clone(), def);
        Ok(StatementRecord::ProcedureDefinition(ident.clone()))
    }

    fn evaluate_procedure_call(
        &mut self,
        procedure_identifier: &str,
    ) -> Result<StatementRecord, CrawlError> {
        let outer_facts = self.local_facts.clone();

        let proc = self.procedures.get(procedure_identifier).unwrap();

        let mut records = Vec::new();
        // How to avoid this clone?
        for statement in proc.body.clone() {
            records.push(Box::new(self.evaluate_statement(&statement)?));
        }

        self.local_facts = outer_facts;
        Ok(StatementRecord::ProcedureCall {
            identifier: procedure_identifier.into(),
            records,
        })
    }

    fn evaluate_check_persistent_fact(&mut self, fact: String) -> Result<bool, CrawlError> {
        Ok(self.persistent_facts.check(&fact.try_into().unwrap()))
    }

    fn evaluate_set_persistent_fact(
        &mut self,
        fact: String,
    ) -> Result<StatementRecord, CrawlError> {
        self.persistent_facts.set(fact.clone().try_into().unwrap());
        Ok(StatementRecord::SetPersistentFact(fact))
    }

    fn evaluate_clear_persistent_fact(
        &mut self,
        fact: String,
    ) -> Result<StatementRecord, CrawlError> {
        self.persistent_facts
            .clear(&fact.clone().try_into().unwrap());
        Ok(StatementRecord::ClearPersistentFact(fact))
    }

    fn evaluate_check_fact(&mut self, fact: String) -> Result<bool, CrawlError> {
        Ok(self.local_facts.check(&fact.try_into().unwrap()))
    }

    fn evaluate_set_fact(&mut self, fact: CrawlStr) -> Result<StatementRecord, CrawlError> {
        let evaluated_fact = self.evaluate_str(fact.clone())?;
        self.local_facts
            .set(evaluated_fact.clone().try_into().unwrap());
        Ok(StatementRecord::SetFact(evaluated_fact))
    }

    fn evaluate_clear_fact(&mut self, fact: String) -> Result<StatementRecord, CrawlError> {
        self.local_facts.clear(&fact.clone().try_into().unwrap());
        Ok(StatementRecord::ClearFact(fact))
    }

    fn evaluate_dice_roll(
        &self,
        target: &Token,
        modified_roll_specifier: &ModifiedRollSpecifier,
    ) -> Result<bool, CrawlError> {
        let roll: DiceRoll = modified_roll_specifier.try_into()?;
        let roll_result = roll.roll();
        self.roll_result_matches_target(&roll_result, target)
    }

    fn evaluate_nontargeted_roll(
        &mut self,
        modified_roll_specifier: &ModifiedRollSpecifier,
    ) -> Result<StatementRecord, CrawlError> {
        let roll: DiceRoll = modified_roll_specifier.try_into()?;
        let roll_result = roll.roll();
        Ok(StatementRecord::NontargetedRoll(roll_result.total))
    }

    fn evaluate_str(&mut self, s: CrawlStr) -> Result<String, CrawlError> {
        match s {
            CrawlStr::Str(raw_string) => Ok(raw_string.clone()),
            CrawlStr::InterpolatedStr {
                format_string,
                expressions,
            } => {
                let re = Regex::new(r"\{.*\}").unwrap();
                let mut replaced: Cow<'_, str> = format_string.clone().into();
                for expr in expressions {
                    replaced = re.replace(
                        &format_string,
                        format!("{:?}", self.evaluate_statement(&expr)?),
                    );
                }

                Ok(replaced.to_string())
            }
        }
    }

    fn roll_result_matches_target(
        &self,
        roll_result: &DiceRollResult,
        target: &Token,
    ) -> Result<bool, CrawlError> {
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
    use crate::facts::Fact;

    use super::*;

    fn interp_to_values(statements: Vec<Statement>) -> Vec<StatementRecord> {
        Interpreter::new()
            .interpret(statements)
            .into_iter()
            .map(|v| v.unwrap())
            .collect()
    }

    fn make_proc_body() -> Vec<Box<Statement>> {
        vec![
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
        ]
    }

    #[test]
    fn interpret_reminder() {
        let ast = Statement::Reminder("players must eat rations daily".into());
        let value = interp_to_values(vec![ast]);
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
        let value = interp_to_values(vec![ast]);
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
        let value = interp_to_values(vec![ast]);
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
        let body = make_proc_body();
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
        assert_eq!(
            *interp.procedures.get("proc-name").unwrap().body,
            body.into_iter().map(|s| *s).collect::<Vec<Statement>>()
        );
    }

    #[test]
    fn interpret_proc_call() {
        let body = make_proc_body();
        let proc = Statement::Procedure {
            declaration: ProcedureDeclaration("proc-name".into()),
            body: body.clone(),
        };
        let call = Statement::ProcedureCall("proc-name".into());
        let ast = vec![proc, call];
        let values = interp_to_values(ast);
        assert_eq!(
            values,
            vec![
                StatementRecord::ProcedureDefinition("proc-name".into()),
                StatementRecord::ProcedureCall {
                    identifier: "proc-name".into(),
                    records: vec![
                        Box::new(StatementRecord::IfThen {
                            antecedent: true,
                            consequent: Some(Box::new(StatementRecord::Reminder(
                                "you passed the check".into()
                            ))),
                        }),
                        Box::new(StatementRecord::Reminder("cool procedure".into())),
                    ],
                },
            ],
        )
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
        let values = interp_to_values(vec![ast]);
        assert_eq!(
            values,
            vec![StatementRecord::MatchingRoll {
                matched_target: Some(Token::Num(1)),
                consequent: Some(Box::new(StatementRecord::Reminder("matched 1".into()))),
            }]
        )
    }

    #[test]
    fn interpret_set_persistent_fact() {
        let ast = Statement::SetPersistentFact("weather is nice".into());
        let mut interp = Interpreter::new();
        let values: Vec<StatementRecord> = interp
            .interpret(vec![ast])
            .into_iter()
            .map(|v| v.unwrap())
            .collect();
        assert_eq!(
            values,
            vec![StatementRecord::SetPersistentFact("weather is nice".into())]
        );
        assert!(interp
            .persistent_facts
            .check(&Fact::try_from(String::from("weather is nice")).unwrap()));
    }

    #[test]
    fn interpret_load_table() {
        let ast = Statement::LoadTable("examples/table.csv".into());
        let mut interp = Interpreter::new();
        let values: Vec<StatementRecord> = interp
            .interpret(vec![ast])
            .into_iter()
            .map(|v| v.unwrap())
            .collect();
        assert_eq!(values, vec![StatementRecord::LoadTable("examples/table.csv".into())]);
        assert!(interp.tables.contains_key("examples/table.csv"));
    }

    #[test]
    fn interpret_table_roll() {
        let ast = vec![
            Statement::LoadTable("examples/table.csv".into()),
            Statement::TableRoll("examples/table.csv".into()),
        ];
        // TODO: not really a test
        let _ = interp_to_values(ast);
    }

    #[test]
    fn interpret_str_interpolation() {
        let ast = vec![Statement::SetFact(CrawlStr::InterpolatedStr {
            format_string: "number is {}".into(),
            expressions: vec![Statement::NontargetedRoll(ModifiedRollSpecifier {
                base_roll_specifier: Token::RollSpecifier("1d1".into()),
                modifier: 0,
            })],
        })];
        let mut interp = Interpreter::new();
        let values: Vec<StatementRecord> = interp
            .interpret(ast)
            .into_iter()
            .map(|v| v.unwrap())
            .collect();
        // TODO: just show the number
        assert_eq!(values, vec![StatementRecord::SetFact("number is NontargetedRoll(1)".into())]);
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
