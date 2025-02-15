use core::fmt;
use std::{cmp::Ordering, collections::HashMap};

use rand::Rng;

#[derive(Debug)]
pub struct DieRollResult(i32);

impl fmt::Display for DieRollResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Die(i32);

impl Die {
    fn roll(&self) -> DieRollResult {
        DieRollResult(rand::thread_rng().gen_range(1..=self.0))
    }
}

#[derive(Debug)]
pub struct DicePoolRollResult {
    pub results: Vec<DieRollResult>,
}

#[derive(Debug)]
pub struct DicePool {
    dice: Vec<Die>,
}

impl DicePool {
    fn roll(&self) -> DicePoolRollResult {
        DicePoolRollResult {
            results: self.dice.iter().map(Die::roll).collect(),
        }
    }
}

impl fmt::Display for DicePool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut counter: HashMap<&Die, usize> = HashMap::new();
        for die in &self.dice {
            *counter.entry(die).or_default() += 1;
        }
        let mut output = String::new();
        for (die_size, count) in counter {
            output.push_str(&format!("{}d{}", count, die_size.0));
        }
        write!(f, "{output}")
    }
}

#[derive(Debug)]
pub struct DiceRollResult {
    pub pool_result: DicePoolRollResult,
    pub modifier: i32,
    pub total: i32,
}

impl fmt::Display for DiceRollResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.total)
    }
}

#[derive(Debug)]
pub struct DiceRoll {
    dice_pool: DicePool,
    modifier: i32,
}

impl DiceRoll {
    pub fn roll(&self) -> DiceRollResult {
        let pool_result = self.dice_pool.roll();
        let unmodified_total = pool_result.results.iter().fold(0, |acc, e| acc + e.0);
        DiceRollResult {
            pool_result,
            modifier: self.modifier,
            total: unmodified_total + self.modifier,
        }
    }
}

impl fmt::Display for DiceRoll {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.modifier.cmp(&0) {
            Ordering::Greater => write!(f, "{} + {}", self.dice_pool, self.modifier),
            Ordering::Less => write!(f, "{} - {}", self.dice_pool, self.modifier.abs()),
            Ordering::Equal => write!(f, "{}", self.dice_pool),
        }
    }
}
