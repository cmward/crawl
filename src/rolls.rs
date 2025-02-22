use crate::error::CrawlError;

#[derive(Clone, Debug, PartialEq)]
pub enum RollTarget {
    Num(i32),
    NumRange(i32, i32),
}

impl TryFrom<String> for RollTarget {
    type Error = CrawlError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let s = value.split('-').collect::<Vec<&str>>();
        match s.len() {
            1 => {
                let num = s
                    .first()
                    .expect("roll target should be a value")
                    .parse::<i32>()
                    .expect("roll target should be a number");
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
