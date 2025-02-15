use crate::error::CrawlError;
use crate::parser::ModifiedRollSpecifier;
use crate::scanner::Token;
use core::fmt;
use rand::Rng;
use regex::Regex;
use std::collections::HashMap;

#[derive(Debug)]
pub struct DieRollResult(i32);

impl fmt::Display for DieRollResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Die(pub i32);

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
    pub dice: Vec<Die>,
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
    pub dice_pool: DicePool,
    pub modifier: i32,
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

impl TryFrom<&ModifiedRollSpecifier> for DiceRoll {
    type Error = CrawlError;

    fn try_from(value: &ModifiedRollSpecifier) -> Result<Self, Self::Error> {
        if let Token::RollSpecifier(ref spec) = value.base_roll_specifier {
            let re = Regex::new(r"(?<n_dice>\d+)*d(?<n_sides>\d+)").unwrap();
            let captures = re
                .captures(&spec)
                .ok_or(CrawlError::ParserError {
                    token: format!("{:?}", value),
                })
                .expect("failed to parse roll specifier");

            let n_dice = captures["n_dice"].parse().expect("failed to parse n_dice");
            let n_sides = captures["n_sides"]
                .parse()
                .expect("failed to parse n_sides");

            let mut dice = Vec::new();
            for _ in 0..n_dice {
                dice.push(Die(n_sides));
            }

            Ok(DiceRoll {
                dice_pool: DicePool { dice },
                modifier: value.modifier,
            })
        } else {
            Err(CrawlError::ParserError {
                token: format!("{:?}", value),
            })
        }
    }
}

impl TryFrom<ModifiedRollSpecifier> for DiceRoll {
    type Error = CrawlError;

    fn try_from(value: ModifiedRollSpecifier) -> Result<Self, Self::Error> {
        Self::try_from(&value)
    }
}
