use std::collections::HashMap;

use crate::{dice::DiceRoll, error::CrawlError, rolls::RollTarget};

#[derive(Clone, Debug, PartialEq)]
pub struct TableEntry {
    roll_target: RollTarget,
    value: String,
}

#[derive(Debug, PartialEq)]
pub struct TableRollResult<'a> {
    entry: &'a TableEntry,
}

impl<'a> TableRollResult<'a> {
    pub fn new(entry: &'a TableEntry) -> Self {
        Self { entry }
    }
}

#[derive(Debug)]
pub struct Table<'a> {
    entries: HashMap<i32, &'a TableEntry>,
}

impl<'a> Table<'a> {
    pub fn new(entries: HashMap<i32, &'a TableEntry>) -> Self {
        Table { entries }
    }

    pub fn roll(&self, dice: DiceRoll) -> Result<TableRollResult, CrawlError> {
        let roll_result = dice.roll();
        if let Some(selected_entry) = self.entries.get(&roll_result.total) {
            Ok(TableRollResult::new(selected_entry))
        } else {
            Err(CrawlError::InterpreterError {
                reason: format!("roll {roll_result:?} not a valid index for table"),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dice::{DicePool, Die};

    use super::*;

    #[test]
    fn roll_num_target() {
        let zero_entry = TableEntry {
            roll_target: RollTarget::Num(0),
            value: "zero".into(),
        };
        let one_entry = TableEntry {
            roll_target: RollTarget::Num(1),
            value: "one".into(),
        };

        let table = Table::new(HashMap::from([(0, &zero_entry), (1, &one_entry)]));

        let dice = DiceRoll::new(DicePool::new(vec![Die(1)]), 0);
        let result = table.roll(dice).unwrap();

        assert_eq!(result, TableRollResult { entry: &one_entry });
    }

    #[test]
    fn roll_range_target() {
        let low_entry = TableEntry {
            roll_target: RollTarget::Num(0),
            value: "1-6".into(),
        };
        let high_entry = TableEntry {
            roll_target: RollTarget::Num(1),
            value: "7-12".into(),
        };
        let table = Table::new(HashMap::from([
            (1, &low_entry),
            (2, &low_entry),
            (3, &low_entry),
            (4, &low_entry),
            (5, &low_entry),
            (6, &low_entry),
            (7, &high_entry),
            (8, &high_entry),
            (9, &high_entry),
            (10, &high_entry),
            (11, &high_entry),
            (12, &high_entry),
        ]));

        let dice = DiceRoll::new(DicePool::new(vec![Die(1)]), 0);
        let result = table.roll(dice).unwrap();
        assert_eq!(result, TableRollResult { entry: &low_entry });

        let dice = DiceRoll::new(DicePool::new(vec![Die(1)]), 11);
        let result = table.roll(dice).unwrap();
        assert_eq!(result, TableRollResult { entry: &high_entry });
    }
}
