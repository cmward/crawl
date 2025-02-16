use std::collections::HashSet;

use crate::error::CrawlError;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Fact {
    entity: String,
    attribute: String,
    value: String,
}

impl TryFrom<String> for Fact {
    type Error = CrawlError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if let Some((entity, tail)) = value.split_once(" ") {
            if let Some((attribute, val)) = tail.split_once(" ") {
                Ok(Fact {
                    entity: entity.into(),
                    attribute: attribute.into(),
                    value: val.into(),
                })
            } else {
                Err(CrawlError::InterpreterError {
                    reason: "couldn't convert to Fact".into(),
                })
            }
        } else {
            Err(CrawlError::InterpreterError {
                reason: "couldn't convert to Fact".into(),
            })
        }
    }
}

#[derive(Clone, Debug)]
pub struct FactDatabase {
    // Making this a HashSet may prove too restrictive in the future. Right now, all we
    // want to do is add and delete triples from a store. If we ever want to query by
    // entities, attributes, or values, we'd want to change this.
    pub facts: HashSet<Fact>,
}

impl Default for FactDatabase {
    fn default() -> Self {
        Self::new(HashSet::new())
    }
}

impl FactDatabase {
    pub fn new(facts: HashSet<Fact>) -> Self {
        FactDatabase { facts }
    }

    pub fn check(&self, fact: &Fact) -> bool {
        self.facts.contains(fact)
    }

    pub fn set(&mut self, fact: Fact) {
        self.facts.insert(fact);
    }

    pub fn clear(&mut self, fact: &Fact) {
        self.facts.remove(fact);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_string() {
        assert_eq!(
            Fact::try_from(String::from("weather is partially cloudy")).unwrap(),
            Fact {
                entity: "weather".into(),
                attribute: "is".into(),
                value: "partially cloudy".into(),
            },
        )
    }
}
