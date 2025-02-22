use std::{collections::HashMap, error::Error, fs::File};

use crate::{dice::DiceRoll, error::CrawlError, rolls::RollTarget};

#[derive(Debug, PartialEq, Clone)]
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
pub struct Table {
    entries: Vec<TableEntry>,
    roll_targets: HashMap<i32, usize>,
}

impl Table {
    pub fn new(entries: Vec<TableEntry>, roll_targets: HashMap<i32, usize>) -> Self {
        Table {
            entries,
            roll_targets,
        }
    }

    pub fn roll(&self, dice: DiceRoll) -> Result<TableRollResult, CrawlError> {
        let roll_result = dice.roll();
        if let Some(entry_idx) = self.roll_targets.get(&roll_result.total) {
            Ok(TableRollResult::new(self.entries.get(*entry_idx).unwrap()))
        } else {
            Err(CrawlError::InterpreterError {
                reason: format!("roll {roll_result:?} not a valid index for table"),
            })
        }
    }

    pub fn load(filepath: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(filepath)?;
        let mut rdr = csv::ReaderBuilder::new()
            .has_headers(false)
            .from_reader(file);

        let mut entries = Vec::new();
        for result in rdr.records() {
            let record = result?;
            let roll_target = record.get(0).unwrap();
            let value = record.get(1).unwrap();
            let entry = TableEntry {
                roll_target: String::from(roll_target).try_into().unwrap(),
                value: value.into(),
            };
            entries.push(entry);
        }

        Ok(Self::from(entries))
    }
}

impl From<Vec<TableEntry>> for Table {
    fn from(value: Vec<TableEntry>) -> Self {
        let mut entries = Vec::new();
        let mut roll_targets = HashMap::<i32, usize>::new();
        for (idx, entry) in value.into_iter().enumerate() {
            let entry_roll_targets = match entry.roll_target {
                RollTarget::Num(n) => vec![n],
                RollTarget::NumRange(n, m) => (n..=m).collect(),
            };
            entries.push(entry);

            for roll_target in entry_roll_targets {
                roll_targets.insert(roll_target, idx);
            }
        }

        Table {
            entries,
            roll_targets,
        }
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
        let table = Table::from(vec![low_entry.clone(), high_entry.clone()]);

        let dice = DiceRoll::new(DicePool::new(vec![Die(1)]), 0);
        let result = table.roll(dice).unwrap();
        assert_eq!(result, TableRollResult { entry: &low_entry });

        let dice = DiceRoll::new(DicePool::new(vec![Die(1)]), 11);
        let result = table.roll(dice).unwrap();
        assert_eq!(result, TableRollResult { entry: &high_entry });
    }

    #[test]
    fn num_target_table_from_vec() {
        let zero_entry = TableEntry {
            roll_target: RollTarget::Num(0),
            value: "zero".into(),
        };
        let one_entry = TableEntry {
            roll_target: RollTarget::Num(1),
            value: "one".into(),
        };

        let table = Table::from(vec![zero_entry.clone(), one_entry.clone()]);

        let dice = DiceRoll::new(DicePool::new(vec![Die(1)]), 0);
        let result = table.roll(dice).unwrap();

        assert_eq!(result, TableRollResult { entry: &one_entry });
    }

    #[test]
    fn from_csv() {
        let table = dbg!(Table::load("table.csv").unwrap());

        let dice = DiceRoll::new(DicePool::new(vec![Die(1)]), 11);
        let result = table.roll(dice).unwrap();

        let entry = TableEntry {
            roll_target: RollTarget::NumRange(7, 12),
            value: "total darkness and dread".into(),
        };
        assert_eq!(result, TableRollResult { entry: &entry })
    }
}
