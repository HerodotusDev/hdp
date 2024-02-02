use anyhow::{bail, Result};
use common::datalake::base::DataPoint;

/// Returns the average of the values
pub fn average(values: &[DataPoint]) -> Result<DataPoint> {
    if values.is_empty() {
        bail!("No values found");
    }

    let mut sum = 0.0;

    for value in values {
        match value {
            DataPoint::Str(_) => bail!("String value found"),
            DataPoint::Int(int) => {
                sum += *int as f64;
            }
            DataPoint::Float(float) => {
                sum += *float;
            }
        }
    }

    Ok(DataPoint::Float(sum / values.len() as f64))
}

pub fn bloom_filterize(_values: &[DataPoint]) -> Result<DataPoint> {
    Ok(DataPoint::Float(0.0))
}

/// Find the maximum value
pub fn find_max(values: &[DataPoint]) -> Result<DataPoint> {
    if values.is_empty() {
        bail!("No values found");
    }

    let mut max = 0.0;

    for value in values {
        match value {
            DataPoint::Str(_) => bail!("String value found"),
            DataPoint::Int(int) => {
                if *int as f64 > max {
                    max = *int as f64;
                }
            }
            DataPoint::Float(float) => {
                if *float > max {
                    max = *float;
                }
            }
        }
    }

    Ok(DataPoint::Float(max))
}

/// Find the minimum value
pub fn find_min(values: &[DataPoint]) -> Result<DataPoint> {
    if values.is_empty() {
        bail!("No values found");
    }

    let mut min = f64::MAX;
    for value in values {
        match value {
            DataPoint::Str(_) => bail!("String value found"),
            DataPoint::Int(int) => {
                if (*int as f64) < min {
                    min = *int as f64;
                }
            }
            DataPoint::Float(float) => {
                if *float < min {
                    min = *float;
                }
            }
        }
    }

    Ok(DataPoint::Float(min))
}

/// Standard deviation
pub fn standard_deviation(values: &[DataPoint]) -> Result<DataPoint> {
    if values.is_empty() {
        bail!("No values found");
    }

    let mut sum = 0.0;
    let count = values.len() as f64;

    for value in values {
        match value {
            DataPoint::Str(_) => bail!("String value found"),
            DataPoint::Int(int) => {
                sum += *int as f64;
            }
            DataPoint::Float(float) => {
                sum += *float;
            }
        }
    }

    let avg = sum / count;

    let mut variance_sum = 0.0;
    for value in values {
        match value {
            DataPoint::Str(_) => bail!("String value found"),
            DataPoint::Int(int) => {
                variance_sum += (*int as f64 - avg).powi(2);
            }
            DataPoint::Float(float) => {
                variance_sum += (*float - avg).powi(2);
            }
        }
    }

    let variance = variance_sum / count;
    Ok(DataPoint::Float(variance.sqrt()))
}

/// Sum of values
pub fn sum(values: &[DataPoint]) -> Result<DataPoint> {
    if values.is_empty() {
        bail!("No values found");
    }

    let mut sum = 0.0;

    for value in values {
        match value {
            DataPoint::Str(_) => bail!("String value found"),
            DataPoint::Int(int) => {
                sum += *int as f64;
            }
            DataPoint::Float(float) => {
                sum += *float;
            }
        }
    }

    Ok(DataPoint::Float(sum))
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
pub fn count_if(values: &[DataPoint], ctx: &str) -> Result<DataPoint> {
    let logical_operator = &ctx[0..2];
    let value_to_compare = &DataPoint::Int(usize::from_str_radix(&ctx[2..], 16).unwrap());

    let mut condition_satisfiability_count = 0;

    for value in values {
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

    Ok(DataPoint::Int(condition_satisfiability_count))
}
