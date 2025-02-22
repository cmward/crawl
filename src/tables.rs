use std::collections::HashMap;

use crate::{dice::DiceRoll, error::CrawlError, rolls::RollTarget};

#[derive(Debug, PartialEq)]
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

impl<'a> From<Vec<&'a TableEntry>> for Table<'a> {
    fn from(value: Vec<&'a TableEntry>) -> Self {
        let mut entries = HashMap::<i32, &'a TableEntry>::new();
        for entry in value {
            let idxs = match entry.roll_target {
                RollTarget::Num(n) => vec![n],
                RollTarget::NumRange(n, m) => (n..=m).collect(),
            };

            for idx in idxs {
                entries.insert(idx, entry);
            }
        }

        Table { entries }
    }
}

#[cfg(test)]
mod tests {
    use crate::dice::{DicePool, Die};

    use super::*;

    #[test]
    fn range_target_table_from_vec() {
        let low_entry = TableEntry {
            roll_target: RollTarget::NumRange(1, 6),
            value: "1-6".into(),
        };
        let high_entry = TableEntry {
            roll_target: RollTarget::NumRange(7, 12),
            value: "7-12".into(),
        };
        let table = dbg!(Table::from(vec![&low_entry, &high_entry]));

        let dice = dbg!(DiceRoll::new(DicePool::new(vec![Die(1)]), 0));
        let result = dbg!(table.roll(dice).unwrap());
        assert_eq!(result, TableRollResult { entry: &low_entry });

        let dice = DiceRoll::new(DicePool::new(vec![Die(1)]), 11);
        let result = table.roll(dice).unwrap();
        assert_eq!(result, TableRollResult { entry: &high_entry });
    }

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
}
