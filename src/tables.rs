use std::{collections::HashMap, error::Error, fs::File};

use crate::{
    dice::{DicePool, DiceRoll, Die},
    error::CrawlError,
    rolls::RollTarget,
};

#[derive(Debug, PartialEq, Clone)]
pub struct TableEntry {
    roll_target: RollTarget,
    pub value: String,
}

#[derive(Debug, PartialEq)]
pub struct TableRollResult<'a> {
    pub entry: &'a TableEntry,
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
    min_target: i32,
    max_target: i32,
    clamp_to_min: bool,
    clamp_to_max: bool,
}

impl Table {
    pub fn new(entries: Vec<TableEntry>, roll_targets: HashMap<i32, usize>) -> Self {
        let min_target = *roll_targets.keys().min().unwrap();
        let max_target = *roll_targets.keys().max().unwrap();
        Table {
            entries,
            roll_targets,
            min_target,
            max_target,
            clamp_to_min: true,
            clamp_to_max: true,
        }
    }

    fn get_value_for_target(&self, target: &i32) -> Result<TableRollResult, CrawlError> {
        if let Some(entry_idx) = self.roll_targets.get(target) {
            Ok(TableRollResult::new(self.entries.get(*entry_idx).unwrap()))
        } else {
            Err(CrawlError::InterpreterError {
                reason: format!("target {target:?} not a valid index for table"),
            })
        }
    }

    pub fn roll(&self, dice: &DiceRoll) -> Result<TableRollResult, CrawlError> {
        let roll_result = dice.roll();
        let roll_value = self.roll_targets.get(&roll_result.total);
        match roll_value {
            Some(entry_idx) => Ok(TableRollResult::new(self.entries.get(*entry_idx).unwrap())),
            None => {
                if roll_result.total < self.min_target && self.clamp_to_min {
                    return self.get_value_for_target(&self.min_target);
                } else if roll_result.total > self.max_target && self.clamp_to_max {
                    return self.get_value_for_target(&self.max_target);
                } else {
                    Err(CrawlError::InterpreterError {
                        reason: format!("roll {roll_result:?} not a valid index for table"),
                    })
                }
            }
        }
    }

    pub fn auto_roll(&self) -> Result<TableRollResult, CrawlError> {
        let dice = vec![Die(self.max_target)];
        let dice_pool = DicePool::new(dice);
        let roll = DiceRoll::new(dice_pool, 0);
        self.roll(&roll)
    }

    // TODO: load from table paths + without extension
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
                roll_target: roll_target.try_into().unwrap(),
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
                RollTarget::OverOrEqual(n) => vec![n],
            };
            entries.push(entry);

            for roll_target in entry_roll_targets {
                roll_targets.insert(roll_target, idx);
            }
        }

        Self::new(entries, roll_targets)
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
        let result = table.roll(&dice).unwrap();
        assert_eq!(result, TableRollResult { entry: &low_entry });

        let dice = DiceRoll::new(DicePool::new(vec![Die(1)]), 11);
        let result = table.roll(&dice).unwrap();
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
        let result = table.roll(&dice).unwrap();

        assert_eq!(result, TableRollResult { entry: &one_entry });
    }

    #[test]
    fn under_min_target() {
        let zero_entry = TableEntry {
            roll_target: RollTarget::Num(0),
            value: "zero".into(),
        };
        let one_entry = TableEntry {
            roll_target: RollTarget::Num(1),
            value: "one".into(),
        };

        let table = Table::from(vec![zero_entry.clone(), one_entry.clone()]);

        let dice = DiceRoll::new(DicePool::new(vec![Die(1)]), -100);
        let result = table.roll(&dice).unwrap();

        assert_eq!(result, TableRollResult { entry: &zero_entry });
    }

    #[test]
    fn over_max_target() {
        let zero_entry = TableEntry {
            roll_target: RollTarget::Num(0),
            value: "zero".into(),
        };
        let one_entry = TableEntry {
            roll_target: RollTarget::Num(1),
            value: "one".into(),
        };

        let table = Table::from(vec![zero_entry.clone(), one_entry.clone()]);

        let dice = DiceRoll::new(DicePool::new(vec![Die(1)]), 100);
        let result = table.roll(&dice).unwrap();

        assert_eq!(result, TableRollResult { entry: &one_entry });
    }

    #[test]
    fn from_csv() {
        // TODO: test fixtures location
        let table = Table::load("examples/table.csv").unwrap();

        let dice = DiceRoll::new(DicePool::new(vec![Die(1)]), 11);
        let result = table.roll(&dice).unwrap();

        let entry = TableEntry {
            roll_target: RollTarget::NumRange(7, 12),
            value: "total darkness and dread".into(),
        };
        assert_eq!(result, TableRollResult { entry: &entry });
    }

    #[test]
    fn over_target_from_csv() {
        let table = Table::load("examples/table.csv").unwrap();

        let dice = DiceRoll::new(DicePool::new(vec![Die(1)]), 100);
        let result = table.roll(&dice).unwrap();

        let entry = TableEntry {
            roll_target: RollTarget::OverOrEqual(13),
            value: "fog seeps up through the dirt".into(),
        };
        assert_eq!(result, TableRollResult { entry: &entry })
    }
}
