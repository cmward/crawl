use crate::error::CrawlError;

// TODO: confusing to have rolls.rs and dice.rs

#[derive(Clone, Debug, PartialEq)]
pub enum RollTarget {
    Num(i32),
    NumRange(i32, i32),
    OverOrEqual(i32),
}

impl TryFrom<&str> for RollTarget {
    type Error = CrawlError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut s = value.split('-').collect::<Vec<&str>>();
        match s.len() {
            1 => {
                let mut is_roll_over = false;
                if let Some(n) = s.last_mut() {
                    if n.ends_with('+') {
                        is_roll_over = true;
                        *n = n.trim_matches('+');
                    }
                }

                let num = s
                    .first()
                    .expect("roll target should be a value")
                    .parse::<i32>()
                    .expect("roll target should be a number");

                if is_roll_over {
                    return Ok(RollTarget::OverOrEqual(num));
                }

                Ok(RollTarget::Num(num))
            }
            2 => {
                let range_min = s
                    .first()
                    .expect("range min should be a value")
                    .parse::<i32>()
                    .expect("range min should be a number");
                let range_max = s
                    .last()
                    .expect("range max should be a value")
                    .parse::<i32>()
                    .expect("range max should be a number");

                Ok(RollTarget::NumRange(range_min, range_max))
            }
            // TODO: not an interpreter error (can happen in scanner or interpreter)
            _ => Err(CrawlError::InterpreterError {
                reason: format!("cannot convert {value:?} to RollTarget"),
            }),
        }
    }
}

impl TryFrom<String> for RollTarget {
    type Error = CrawlError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}
