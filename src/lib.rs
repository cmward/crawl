use core::fmt;
use std::{cmp::Ordering, collections::HashMap, str::FromStr};

use rand::Rng;
use regex::Regex;

#[derive(Debug)]
struct DieRollResult {
    die: Die,
    throw: i32,
}

impl fmt::Display for DieRollResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.throw)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Die(i32);

impl Die {
    fn roll(&self) -> DieRollResult {
        DieRollResult {
            die: self.clone(),
            throw: rand::thread_rng().gen_range(1..=self.0),
        }
    }
}

#[derive(Debug)]
struct DicePoolRollResult {
    results: Vec<DieRollResult>,
}

#[derive(Debug)]
struct DicePool {
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
    pool_result: DicePoolRollResult,
    modifier: i32,
    total: i32,
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
        let unmodified_total = pool_result.results.iter().fold(0, |acc, e| acc + e.throw);
        DiceRollResult {
            pool_result,
            modifier: self.modifier,
            total: unmodified_total + self.modifier,
        }
    }

    fn parse_n_dice_from_str(
        s: &str,
        captures: &regex::Captures,
    ) -> Result<i32, ParseDiceRollError> {
        captures["n_dice"].parse().map_err(|_| ParseDiceRollError {
            input: String::from(s),
            failed_on: String::from("n_dice"),
        })
    }

    fn parse_n_sides_from_str(
        s: &str,
        captures: &regex::Captures,
    ) -> Result<i32, ParseDiceRollError> {
        captures["n_sides"].parse().map_err(|_| ParseDiceRollError {
            input: String::from(s),
            failed_on: String::from("n_sides"),
        })
    }

    fn parse_modifier_from_str(
        s: &str,
        captures: &regex::Captures,
    ) -> Result<i32, ParseDiceRollError> {
        match &captures.name("modifier") {
            Some(m) => {
                let mut split_modifier = m.as_str().split_whitespace();
                let modifier = match (
                    split_modifier.next(),
                    split_modifier.next(),
                    split_modifier.next(),
                ) {
                    (Some("+"), Some(amount), None) => amount.parse::<i32>().unwrap(),
                    (Some("-"), Some(amount), None) => -amount.parse::<i32>().unwrap(),
                    _ => 0,
                };
                Ok(modifier)
            }
            _ => Err(ParseDiceRollError {
                input: String::from(s),
                failed_on: String::from("modifier"),
            }),
        }
    }

    fn parse_str(s: &str) -> Result<DiceRoll, ParseDiceRollError> {
        let re = Regex::new(r"(?<n_dice>\d+)*d(?<n_sides>\d+)(?<modifier> [+-] \d+)*").unwrap();
        let captures = re.captures(s).ok_or(ParseDiceRollError {
            input: String::from(s),
            failed_on: String::from("capture"),
        })?;

        let n_dice = Self::parse_n_dice_from_str(s, &captures)?;
        let n_sides = Self::parse_n_sides_from_str(s, &captures)?;
        let modifier = Self::parse_modifier_from_str(s, &captures)?;

        let mut dice = vec![];
        for _ in 0..n_dice {
            dice.push(Die(n_sides));
        }

        Ok(DiceRoll {
            dice_pool: DicePool { dice },
            modifier,
        })
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

#[derive(Debug, PartialEq, Eq)]
pub struct ParseDiceRollError {
    input: String,
    failed_on: String,
}

impl FromStr for DiceRoll {
    type Err = ParseDiceRollError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_str(s)
    }
}
