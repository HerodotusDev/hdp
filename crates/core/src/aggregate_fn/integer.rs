use std::str::FromStr;

use alloy_primitives::U256;
use anyhow::{bail, Result};

use super::FunctionContext;

/// Returns the average of the values: [`AVG`](https://en.wikipedia.org/wiki/Average)
pub fn average(values: &[U256]) -> Result<String> {
    if values.is_empty() {
        bail!("No values found");
    }

    let sum = values
        .iter()
        .try_fold(U256::from(0), |acc, val| acc.checked_add(*val))
        .unwrap();

    let divided_value = divide(sum, U256::from(values.len()));

    Ok(divided_value)
}

// TODO: Implement bloom_filterize
pub fn bloom_filterize(_values: &[U256]) -> Result<String> {
    Ok("0".to_string())
}

/// Find the maximum value: [`MAX`](https://en.wikipedia.org/wiki/Maxima_and_minima)
pub fn find_max(values: &[U256]) -> Result<String> {
    if values.is_empty() {
        bail!("No values found");
    }

    let mut max = U256::from(0);

    for value in values {
        if value > &max {
            max = *value;
        }
    }

    Ok(max.to_string())
}

/// Find the minimum value: [`MIN`](https://en.wikipedia.org/wiki/Maxima_and_minima)
pub fn find_min(values: &[U256]) -> Result<String> {
    if values.is_empty() {
        bail!("No values found");
    }

    let mut min = U256::MAX;
    for value in values {
        if value < &min {
            min = *value;
        }
    }

    Ok(min.to_string())
}

/// Standard deviation
/// wip
pub fn standard_deviation(values: &[U256]) -> Result<String> {
    if values.is_empty() {
        bail!("No values found");
    }

    let mut sum = U256::from(0);
    let count = U256::from(values.len());

    for value in values {
        sum += value;
    }

    let avg = divide(sum, count).parse::<f64>().unwrap();

    let mut variance_sum = 0.0;
    for value in values {
        let value = value.to_string().parse::<f64>().unwrap();
        variance_sum += (value - avg).powi(2);
    }

    let variance: f64 = divide(U256::from(variance_sum), U256::from(count))
        .parse()
        .unwrap();
    Ok(roundup(variance.sqrt().to_string()).to_string())
}

/// Sum of values: [`SUM`](https://en.wikipedia.org/wiki/Summation)
pub fn sum(values: &[U256]) -> Result<String> {
    if values.is_empty() {
        bail!("No values found");
    }

    let mut sum = U256::from(0);

    for value in values {
        sum += value;
    }

    Ok(sum.to_string())
}

/// Count number of values that satisfy a condition
///
/// The context is a string of 4 characters:
/// - The first two characters are the logical operator
/// - The last two characters are the value to compare
///
/// The logical operators are:
/// - 00: Equal (=)
/// - 01: Not equal (!=)
/// - 02: Greater than (>)
/// - 03: Greater than or equal (>=)
/// - 04: Less than (<)
/// - 05: Less than or equal (<=)
pub fn count(values: &[U256], ctx: &FunctionContext) -> Result<String> {
    let logical_operator = &ctx.operator;
    let value_to_compare = ctx.value_to_compare;

    let mut condition_satisfiability_count = 0;

    for value in values {
        match logical_operator {
            Operator::Equal => {
                if value == &value_to_compare {
                    condition_satisfiability_count += 1;
                }
            }
            Operator::NotEqual => {
                if value != &value_to_compare {
                    condition_satisfiability_count += 1;
                }
            }
            Operator::GreaterThan => {
                if value > &value_to_compare {
                    condition_satisfiability_count += 1;
                }
            }
            Operator::GreaterThanOrEqual => {
                if value >= &value_to_compare {
                    condition_satisfiability_count += 1;
                }
            }
            Operator::LessThan => {
                if value < &value_to_compare {
                    condition_satisfiability_count += 1;
                }
            }
            Operator::LessThanOrEqual => {
                if value <= &value_to_compare {
                    condition_satisfiability_count += 1;
                }
            }
            Operator::None => {
                bail!("Count need logical operator");
            }
        }
    }

    Ok(condition_satisfiability_count.to_string())
}

pub fn simple_linear_regression(_values: &[U256]) -> Result<String> {
    // TODO: handle custom compute module
    Ok("0".to_string())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operator {
    None,
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

impl FromStr for Operator {
    type Err = anyhow::Error;

    fn from_str(operator: &str) -> Result<Self> {
        match operator {
            "eq" => Ok(Self::Equal),
            "nq" => Ok(Self::NotEqual),
            "gt" => Ok(Self::GreaterThan),
            "gteq" => Ok(Self::GreaterThanOrEqual),
            "lt" => Ok(Self::LessThan),
            "lteq=" => Ok(Self::LessThanOrEqual),
            "none" => Ok(Self::None),
            _ => bail!("Unknown logical operator"),
        }
    }
}

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let operator = match self {
            Operator::Equal => "eq",
            Operator::NotEqual => "nq",
            Operator::GreaterThan => "gt",
            Operator::GreaterThanOrEqual => "gteq",
            Operator::LessThan => "lt",
            Operator::LessThanOrEqual => "lteq",
            Operator::None => "none",
        };
        write!(f, "{}", operator)
    }
}

impl Operator {
    pub fn from_symbol(symbol: &str) -> Result<Self> {
        match symbol {
            "=" => Ok(Self::Equal),
            "!=" => Ok(Self::NotEqual),
            ">" => Ok(Self::GreaterThan),
            ">=" => Ok(Self::GreaterThanOrEqual),
            "<" => Ok(Self::LessThan),
            "<=" => Ok(Self::LessThanOrEqual),
            "none" => Ok(Self::None),
            _ => bail!("Unknown logical operator"),
        }
    }
    // Convert operator to bytes
    pub fn to_index(operator: &Self) -> u8 {
        match operator {
            Operator::Equal => 1,
            Operator::NotEqual => 2,
            Operator::GreaterThan => 3,
            Operator::GreaterThanOrEqual => 4,
            Operator::LessThan => 5,
            Operator::LessThanOrEqual => 6,
            Operator::None => 0,
        }
    }

    pub fn from_index(bytes: u8) -> Result<Self> {
        match bytes {
            0 => Ok(Operator::None),
            1 => Ok(Operator::Equal),
            2 => Ok(Operator::NotEqual),
            3 => Ok(Operator::GreaterThan),
            4 => Ok(Operator::GreaterThanOrEqual),
            5 => Ok(Operator::LessThan),
            6 => Ok(Operator::LessThanOrEqual),
            _ => bail!("Unknown logical operator"),
        }
    }
}

// Handle division properly using U256 type
fn divide(a: U256, b: U256) -> String {
    if b.is_zero() {
        return "Division by zero error".to_string();
    }

    let quotient = a / b;
    let remainder = a % b;
    let divisor_half = b / U256::from(2);

    if remainder > divisor_half || (remainder == divisor_half && b % U256::from(2) == U256::from(0))
    {
        (quotient + U256::from(1)).to_string()
    } else {
        quotient.to_string()
    }
}

fn roundup(value: String) -> u128 {
    let result: f64 = value.parse().unwrap();
    result.round() as u128
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_avg() {
        let values = vec![U256::from(1), U256::from(2), U256::from(3)];
        assert_eq!(average(&values).unwrap(), "2".to_string());

        let values = vec![U256::from(1), U256::from(2)];
        assert_eq!(average(&values).unwrap(), "2".to_string());

        let values = vec![U256::from_str("1000000000000").unwrap()];
        assert_eq!(average(&values).unwrap(), "1000000000000".to_string());

        let values = vec![U256::from_str("41697298409483537348").unwrap()];
        assert_eq!(
            average(&values).unwrap(),
            "41697298409483537348".to_string()
        );
    }

    #[test]
    fn test_sum() {
        let values = vec![U256::from(1), U256::from(2), U256::from(3)];
        assert_eq!(sum(&values).unwrap(), "6".to_string());

        let values = vec![U256::from(1), U256::from(2)];
        assert_eq!(sum(&values).unwrap(), "3".to_string());

        let values = vec![U256::from_str("6776").unwrap()];
        assert_eq!(sum(&values).unwrap(), "6776".to_string());

        let values = vec![U256::from_str("41697298409483537348").unwrap()];
        assert_eq!(sum(&values).unwrap(), "41697298409483537348".to_string());
    }

    #[test]
    fn test_avg_multi() {
        let values = vec![
            U256::from_str("41697095938570171564").unwrap(),
            U256::from_str("41697095938570171564").unwrap(),
            U256::from_str("41697095938570171564").unwrap(),
            U256::from_str("41697095938570171564").unwrap(),
            U256::from_str("41697095938570171564").unwrap(),
            U256::from_str("41697095938570171564").unwrap(),
            U256::from_str("41697095938570171564").unwrap(),
            U256::from_str("41697095938570171564").unwrap(),
            U256::from_str("41697298409483537348").unwrap(),
            U256::from_str("41697298409483537348").unwrap(),
            U256::from_str("41697298409483537348").unwrap(),
        ];
        assert_eq!(
            average(&values).unwrap(),
            "41697151157910180414".to_string()
        );
    }

    #[test]
    fn test_avg_empty() {
        let values = vec![];
        assert!(average(&values).is_err());
    }

    #[test]
    fn test_find_max() {
        let values = vec![U256::from(1), U256::from(2), U256::from(3)];
        assert_eq!(find_max(&values).unwrap(), "3".to_string());

        let values = vec![U256::from(1), U256::from(2)];
        assert_eq!(find_max(&values).unwrap(), "2".to_string());
    }

    #[test]
    fn test_find_min() {
        let values = vec![U256::from(1), U256::from(2), U256::from(3)];
        assert_eq!(find_min(&values).unwrap(), "1".to_string());

        let values = vec![U256::from(1), U256::from(2)];
        assert_eq!(find_min(&values).unwrap(), "1".to_string());
    }

    #[test]
    fn test_std() {
        let values = vec![U256::from(1), U256::from(2), U256::from(3)];
        assert_eq!(standard_deviation(&values).unwrap(), "1".to_string());

        let values = vec![
            U256::from(0),
            U256::from(2),
            U256::from(10),
            U256::from(2),
            U256::from(100),
        ];
        assert_eq!(standard_deviation(&values).unwrap(), "39".to_string());
    }

    #[test]
    fn test_count() {
        let values = vec![U256::from(1), U256::from(165), U256::from(3)];
        //    assert_eq!(count(&values, "04a5").unwrap(), "2".to_string());
        assert_eq!(
            count(
                &values,
                &FunctionContext::new(Operator::GreaterThanOrEqual, U256::from(2))
            )
            .unwrap(),
            "2".to_string()
        );

        let values = vec![U256::from(1), U256::from(10)];
        //assert_eq!(count(&values, "0000000000a").unwrap(), "1".to_string());
        assert_eq!(
            count(
                &values,
                &FunctionContext::new(Operator::GreaterThan, U256::from(1))
            )
            .unwrap(),
            "1".to_string()
        );
    }
}
