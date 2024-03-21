use std::str::FromStr;

use alloy_primitives::U256;
use anyhow::{bail, Result};

pub mod integer;
pub mod string;

/// Aggregation function types
///
/// ### Defined
/// - AVG - Returns the average of the values
/// - BLOOM - Bloom filter
/// - MAX - Find the maximum value
/// - MIN - Find the minimum value
/// - MERKLE - Return the merkle root of the values
/// - STD - Standard deviation
/// - SUM - Sum of values
/// - COUNTIF - Count number of values that satisfy a condition
pub enum AggregationFunction {
    AVG,
    BLOOM,
    MAX,
    MIN,
    MERKLE,
    STD,
    SUM,
    COUNTIF,
}

/// Get [`AggregationFunction`] from function id
impl FromStr for AggregationFunction {
    type Err = anyhow::Error;

    fn from_str(function_id: &str) -> Result<Self, Self::Err> {
        match function_id.to_uppercase().as_str() {
            "AVG" => Ok(AggregationFunction::AVG),
            "BLOOM" => Ok(AggregationFunction::BLOOM),
            "MAX" => Ok(AggregationFunction::MAX),
            "MIN" => Ok(AggregationFunction::MIN),
            "MERKLE" => Ok(AggregationFunction::MERKLE),
            "STD" => Ok(AggregationFunction::STD),
            "SUM" => Ok(AggregationFunction::SUM),
            "COUNTIF" => Ok(AggregationFunction::COUNTIF),
            _ => bail!("Unknown aggregation function"),
        }
    }
}

impl AggregationFunction {
    pub fn get_index(&self) -> u8 {
        match self {
            AggregationFunction::AVG => 0,
            AggregationFunction::BLOOM => 1,
            AggregationFunction::MAX => 2,
            AggregationFunction::MIN => 3,
            AggregationFunction::MERKLE => 4,
            AggregationFunction::STD => 5,
            AggregationFunction::SUM => 6,
            AggregationFunction::COUNTIF => 7,
        }
    }

    pub fn operation(&self, values: &[String], ctx: Option<String>) -> Result<String> {
        match self {
            // Aggregation functions for integer values
            AggregationFunction::AVG => integer::average(&parse_int_value(values).unwrap()),
            AggregationFunction::BLOOM => {
                integer::bloom_filterize(&parse_int_value(values).unwrap())
            }
            AggregationFunction::MAX => integer::find_max(&parse_int_value(values).unwrap()),
            AggregationFunction::MIN => integer::find_min(&parse_int_value(values).unwrap()),
            AggregationFunction::STD => {
                integer::standard_deviation(&parse_int_value(values).unwrap())
            }
            AggregationFunction::SUM => integer::sum(&parse_int_value(values).unwrap()),
            AggregationFunction::COUNTIF => {
                if let Some(ctx) = ctx {
                    integer::count_if(&parse_int_value(values).unwrap(), &ctx)
                } else {
                    bail!("Context not provided for COUNTIF")
                }
            }
            // Aggregation functions for string values
            AggregationFunction::MERKLE => string::merkleize(values),
        }
    }
}

// Remove the "0x" prefix if exist, so that integer functions can parse integer values
// In case of storage value, either if this is number or hex string type, all stored in hex string format.
// So, we need to remove the "0x" prefix to parse the integer value if user target to use integer functions.
// If the value is already in integer format, then it will be parsed as integer, which is decimal format.
//
// This also implies, even if the value is in hex string format, it will be parsed as integer, which is decimal format.
// So for user it's importantant to know the value type and the function type.
fn parse_int_value(values: &[String]) -> Result<Vec<U256>> {
    let int_values: Vec<U256> = values
        .iter()
        .map(|hex_str| {
            if hex_str.starts_with("0x") {
                let hex_value = hex_str.trim_start_matches("0x").to_string();
                U256::from_str_radix(&hex_value, 16).unwrap()
            } else {
                U256::from_str_radix(hex_str, 10).unwrap()
            }
        })
        .collect();

    Ok(int_values)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum() {
        let sum_fn = AggregationFunction::SUM;

        // 4952100 ~ 4952100, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.nonce
        let values = vec!["6776".to_string()];
        let result = sum_fn.operation(&values, None).unwrap();
        assert_eq!(result, "6776");

        // 4952100 ~ 4952103, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.nonce
        let values = vec![
            "6776".to_string(),
            "6776".to_string(),
            "6776".to_string(),
            "6777".to_string(),
        ];
        let result = sum_fn.operation(&values, None).unwrap();
        assert_eq!(result, "27105");

        // 5382810 ~ 5382810, storage.0x75CeC1db9dCeb703200EAa6595f66885C962B920.0x0000000000000000000000000000000000000000000000000000000000000002
        let values = vec!["0x9184e72a000".to_string()];
        let result = sum_fn.operation(&values, None).unwrap();
        assert_eq!(result, "10000000000000");

        // 5382810 ~ 5382813, storage.0x75CeC1db9dCeb703200EAa6595f66885C962B920.0x0000000000000000000000000000000000000000000000000000000000000002
        let values = vec![
            "0x9184e72a000".to_string(),
            "0x9184e72a000".to_string(),
            "0x9184e72a000".to_string(),
            "0x9184e72a000".to_string(),
        ];
        let result = sum_fn.operation(&values, None).unwrap();
        assert_eq!(result, "40000000000000");

        // 4952100 ~ 4952103, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.balance
        let values = vec![
            "41697298409483537348".to_string(),
            "41697298409483537348".to_string(),
            "41697298409483537348".to_string(),
            "41697095938570171564".to_string(),
        ];
        let result = sum_fn.operation(&values, None).unwrap();
        assert_eq!(result, "166788991167020783608");
    }

    #[test]
    fn test_avg() {
        let avg_fn = AggregationFunction::AVG;

        // 4952100 ~ 4952100, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.nonce
        let values = vec!["6776".to_string()];
        let result = avg_fn.operation(&values, None).unwrap();
        assert_eq!(result, "6776");

        // 4952100 ~ 4952110, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.nonce
        let values = vec![
            "6776".to_string(),
            "6776".to_string(),
            "6776".to_string(),
            "6777".to_string(),
            "6777".to_string(),
            "6777".to_string(),
            "6777".to_string(),
            "6777".to_string(),
            "6777".to_string(),
            "6777".to_string(),
            "6777".to_string(),
        ];
        let result = avg_fn.operation(&values, None).unwrap();
        assert_eq!(result, "6777");

        // 5382810 ~ 5382810, storage.0x75CeC1db9dCeb703200EAa6595f66885C962B920.0x0000000000000000000000000000000000000000000000000000000000000002
        let values = vec!["0x9184e72a000".to_string()];
        let result = avg_fn.operation(&values, None).unwrap();
        assert_eq!(result, "10000000000000");

        // 5382810 ~ 5382813, storage.0x75CeC1db9dCeb703200EAa6595f66885C962B920.0x0000000000000000000000000000000000000000000000000000000000000002
        let values = vec![
            "0x9184e72a000".to_string(),
            "0x9184e72a000".to_string(),
            "0x9184e72a000".to_string(),
            "0x9184e72a000".to_string(),
        ];
        let result = avg_fn.operation(&values, None).unwrap();
        assert_eq!(result, "10000000000000");

        // 4952100 ~ 4952110, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.balance
        let values = vec![
            "41697298409483537348".to_string(),
            "41697298409483537348".to_string(),
            "41697298409483537348".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
        ];
        let result = avg_fn.operation(&values, None).unwrap();
        assert_eq!(result, "41697151157910180414");
    }

    #[test]
    fn test_max() {
        let max_fn = AggregationFunction::MAX;

        // 4952100 ~ 4952100, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.nonce
        let values = vec!["6776".to_string()];
        let result = max_fn.operation(&values, None).unwrap();
        assert_eq!(result, "6776");

        // 4952100 ~ 4952110, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.nonce
        let values = vec![
            "6776".to_string(),
            "6776".to_string(),
            "6776".to_string(),
            "6777".to_string(),
            "6777".to_string(),
            "6777".to_string(),
            "6777".to_string(),
            "6777".to_string(),
            "6777".to_string(),
            "6777".to_string(),
            "6777".to_string(),
        ];
        let result = max_fn.operation(&values, None).unwrap();
        assert_eq!(result, "6777");

        // 5382810 ~ 5382810, storage.0x75CeC1db9dCeb703200EAa6595f66885C962B920.0x0000000000000000000000000000000000000000000000000000000000000002
        let values = vec!["0x9184e72a000".to_string()];
        let result = max_fn.operation(&values, None).unwrap();
        assert_eq!(result, "10000000000000");

        // 5382810 ~ 5382813, storage.0x75CeC1db9dCeb703200EAa6595f66885C962B920.0x0000000000000000000000000000000000000000000000000000000000000002
        let values = vec![
            "0x9184e72a000".to_string(),
            "0x9184e72a000".to_string(),
            "0x9184e72a000".to_string(),
            "0x9184e72a000".to_string(),
        ];
        let result = max_fn.operation(&values, None).unwrap();
        assert_eq!(result, "10000000000000");

        // 4952100 ~ 4952110, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.balance
        let values = vec![
            "41697298409483537348".to_string(),
            "41697298409483537348".to_string(),
            "41697298409483537348".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
        ];
        let result = max_fn.operation(&values, None).unwrap();
        assert_eq!(result, "41697298409483537348");
    }

    #[test]
    fn test_min() {
        let min_fn = AggregationFunction::MIN;

        // 4952100 ~ 4952100, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.nonce
        let values = vec!["6776".to_string()];
        let result = min_fn.operation(&values, None).unwrap();
        assert_eq!(result, "6776");

        // 4952100 ~ 4952110, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.nonce
        let values = vec![
            "6776".to_string(),
            "6776".to_string(),
            "6776".to_string(),
            "6777".to_string(),
            "6777".to_string(),
            "6777".to_string(),
            "6777".to_string(),
            "6777".to_string(),
            "6777".to_string(),
            "6777".to_string(),
            "6777".to_string(),
        ];
        let result = min_fn.operation(&values, None).unwrap();
        assert_eq!(result, "6776");

        // 5382810 ~ 5382810, storage.0x75CeC1db9dCeb703200EAa6595f66885C962B920.0x0000000000000000000000000000000000000000000000000000000000000002
        let values = vec!["0x9184e72a000".to_string()];
        let result = min_fn.operation(&values, None).unwrap();
        assert_eq!(result, "10000000000000");

        // 5382810 ~ 5382813, storage.0x75CeC1db9dCeb703200EAa6595f66885C962B920.0x0000000000000000000000000000000000000000000000000000000000000002
        let values = vec![
            "0x9184e72a000".to_string(),
            "0x9184e72a000".to_string(),
            "0x9184e72a000".to_string(),
            "0x9184e72a000".to_string(),
        ];
        let result = min_fn.operation(&values, None).unwrap();
        assert_eq!(result, "10000000000000");

        // 4952100 ~ 4952110, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.balance
        let values = vec![
            "41697298409483537348".to_string(),
            "41697298409483537348".to_string(),
            "41697298409483537348".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
        ];
        let result = min_fn.operation(&values, None).unwrap();
        assert_eq!(result, "41697095938570171564");
    }

    #[test]
    fn test_countif() {
        let count_if = AggregationFunction::COUNTIF;

        // 4952100 ~ 4952100, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.nonce
        let values = vec!["6776".to_string()];
        // logical_operator: 03 (>=)
        // value_to_compare: 0x0000000000000000000000000000000000000000000000000000000000000fff (4095)
        let result = count_if.operation(&values, Some("03fff".into())).unwrap();
        assert_eq!(result, "1");
        // logical_operator: 00 (=)
        // value_to_compare: 0x0000000000000000000000000000000000000000000000000000000000001A78 (6776)
        let result = count_if.operation(&values, Some("001A78".into())).unwrap();
        assert_eq!(result, "1");

        // 4952100 ~ 4952110, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.nonce
        let values = vec![
            "6776".to_string(),
            "6776".to_string(),
            "6776".to_string(),
            "6777".to_string(),
            "6777".to_string(),
            "6777".to_string(),
            "6777".to_string(),
            "6777".to_string(),
            "6777".to_string(),
            "6777".to_string(),
            "6777".to_string(),
        ];
        // logical_operator: 01 (!=)
        // value_to_compare: 0x0000000000000000000000000000000000000000000000000000000000001A78 (6776)
        let result = count_if.operation(&values, Some("011A78".into())).unwrap();
        assert_eq!(result, "8");

        // logical_operator: 02 (>)
        // value_to_compare: 0x0000000000000000000000000000000000000000000000000000000000001A78 (6776)
        let result = count_if.operation(&values, Some("021A78".into())).unwrap();
        assert_eq!(result, "8");

        // 5382810 ~ 5382810, storage.0x75CeC1db9dCeb703200EAa6595f66885C962B920.0x0000000000000000000000000000000000000000000000000000000000000002
        let values = vec!["0x9184e72a000".to_string()];
        // logical_operator: 00 (=)
        // value_to_compare: 0x000000000000000000000000000000000000000000000000000009184e72a000 (10000000000000)
        let result = count_if
            .operation(&values, Some("009184e72a000".into()))
            .unwrap();
        assert_eq!(result, "1");

        // 5382810 ~ 5382813, storage.0x75CeC1db9dCeb703200EAa6595f66885C962B920.0x0000000000000000000000000000000000000000000000000000000000000002
        // logical_operator: 05 (<=)
        // value_to_compare: 0x000000000000000000000000000000000000000000000000000009184e72a001 (10000000000001)
        let values = vec![
            "0x9184e72a000".to_string(),
            "0x9184e72a000".to_string(),
            "0x9184e72a000".to_string(),
            "0x9184e72a000".to_string(),
        ];
        let result = count_if
            .operation(&values, Some("059184e72a001".into()))
            .unwrap();
        assert_eq!(result, "4");

        // 4952100 ~ 4952110, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.balance
        // logical_operator: 05 (<=)
        // value_to_compare: 0x00000000000000000000000000000000000000000000000242a9d7d5dfdbb4ac (41697095938570171564)
        let values = vec![
            "41697298409483537348".to_string(),
            "41697298409483537348".to_string(),
            "41697298409483537348".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
            "41697095938570171564".to_string(),
        ];
        let result = count_if
            .operation(&values, Some("05242A9D7D5DFDBB4AC".into()))
            .unwrap();
        assert_eq!(result, "8");
    }
}
