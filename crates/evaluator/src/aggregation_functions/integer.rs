use alloy_primitives::U256;
use anyhow::{bail, Result};

/// Returns the average of the values
pub fn average(values: &[String]) -> Result<String> {
    if values.is_empty() {
        bail!("No values found");
    }

    let mut sum = 0;

    for value in values {
        let value = value.parse::<u128>()?;
        sum += value;
    }

    let divided_value = divide(sum, values.len() as u128);

    Ok(divided_value)
}

// TODO: Implement bloom_filterize
pub fn bloom_filterize(_values: &[String]) -> Result<String> {
    Ok("0".to_string())
}

/// Find the maximum value
pub fn find_max(values: &[String]) -> Result<String> {
    if values.is_empty() {
        bail!("No values found");
    }

    let mut max = 0;

    for value in values {
        let value = value.parse::<u64>()?;

        if value > max {
            max = value;
        }
    }

    Ok(max.to_string())
}

/// Find the minimum value
pub fn find_min(values: &[String]) -> Result<String> {
    if values.is_empty() {
        bail!("No values found");
    }

    let mut min = u64::MAX;
    for value in values {
        let value = value.parse::<u64>()?;

        if value < min {
            min = value;
        }
    }

    Ok(min.to_string())
}

/// Standard deviation
pub fn standard_deviation(values: &[String]) -> Result<String> {
    if values.is_empty() {
        bail!("No values found");
    }

    let mut sum = 0.0;
    let count = values.len() as f64;

    for value in values {
        let value = value.parse::<f64>()?;
        sum += value;
    }

    let avg = sum / count;

    let mut variance_sum = 0.0;
    for value in values {
        let value = value.parse::<f64>()?;
        variance_sum += (value - avg).powi(2);
    }

    let variance: f64 = divide(variance_sum as u128, count as u128).parse().unwrap();
    Ok(roundup(variance.sqrt().to_string()).to_string())
}

/// Sum of values
pub fn sum(values: &[String]) -> Result<String> {
    if values.is_empty() {
        bail!("No values found");
    }

    let mut sum = 0;

    for value in values {
        let value = value.parse::<u128>()?;
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
/// - 00: Equal
/// - 01: Not equal
/// - 02: Greater than
/// - 03: Greater than or equal
/// - 04: Less than
/// - 05: Less than or equal
pub fn count_if(values: &[String], ctx: &str) -> Result<String> {
    let logical_operator = &ctx[0..2];
    let value_to_compare = u64::from_str_radix(&ctx[2..], 16).unwrap();

    let mut condition_satisfiability_count = 0;

    for value in values {
        let value = value.parse::<u64>()?;
        match logical_operator {
            "00" => {
                if value == value_to_compare {
                    condition_satisfiability_count += 1;
                }
            }
            "01" => {
                if value != value_to_compare {
                    condition_satisfiability_count += 1;
                }
            }
            "02" => {
                if value > value_to_compare {
                    condition_satisfiability_count += 1;
                }
            }
            "03" => {
                if value >= value_to_compare {
                    condition_satisfiability_count += 1;
                }
            }
            "04" => {
                if value < value_to_compare {
                    condition_satisfiability_count += 1;
                }
            }
            "05" => {
                if value <= value_to_compare {
                    condition_satisfiability_count += 1;
                }
            }
            _ => bail!("Unknown logical operator"),
        }
    }

    Ok(condition_satisfiability_count.to_string())
}

fn divide(a: u128, b: u128) -> String {
    let a_u256 = U256::from(a);
    let b_u256 = U256::from(b);
    if b_u256.is_zero() {
        return "Division by zero error".to_string();
    }

    let quotient = a_u256 / b_u256;
    let remainder = a_u256 % b_u256;
    let divisor_half = b_u256 / U256::from(2);

    if remainder > divisor_half
        || (remainder == divisor_half && b_u256 % U256::from(2) == U256::from(0))
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
