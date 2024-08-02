use alloy::primitives::U256;
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use self::integer::Operator;

pub mod integer;

/// Aggregation function types
///
/// ### Defined
/// - AVG - Returns the average of the values
/// - SUM - Sum of values
/// - MIN - Find the minimum value
/// - MAX - Find the maximum value
/// - COUNT - Count number of values that satisfy a condition
/// - MERKLE - Return the merkle root of the values
/// - SLR - Simple Linear Regression
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AggregationFunction {
    AVG,
    SUM,
    MIN,
    MAX,
    COUNT,
    MERKLE,
    SLR,
}

/// Get [`AggregationFunction`] from function id
impl FromStr for AggregationFunction {
    type Err = anyhow::Error;

    fn from_str(function_id: &str) -> Result<Self, Self::Err> {
        match function_id.to_uppercase().as_str() {
            "AVG" => Ok(Self::AVG),
            "SUM" => Ok(Self::SUM),
            "MIN" => Ok(Self::MIN),
            "MAX" => Ok(Self::MAX),
            "COUNT" => Ok(Self::COUNT),
            "MERKLE" => Ok(Self::MERKLE),
            "SLR" => Ok(Self::SLR),
            _ => bail!("Unknown aggregation function"),
        }
    }
}

impl std::fmt::Display for AggregationFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AggregationFunction::AVG => write!(f, "avg"),
            AggregationFunction::SUM => write!(f, "sum"),
            AggregationFunction::MIN => write!(f, "min"),
            AggregationFunction::MAX => write!(f, "max"),
            AggregationFunction::COUNT => write!(f, "count"),
            AggregationFunction::MERKLE => write!(f, "merkle"),
            AggregationFunction::SLR => write!(f, "slr"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionContext {
    pub operator: Operator,
    pub value_to_compare: U256,
}

impl Default for FunctionContext {
    fn default() -> Self {
        Self {
            operator: Operator::None,
            value_to_compare: U256::ZERO,
        }
    }
}

impl FromStr for FunctionContext {
    type Err = anyhow::Error;

    fn from_str(context: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = context.split('.').collect();
        if parts.len() != 2 {
            bail!("Invalid FnContext format");
        }
        let operator = parts[0].to_string();
        let value_to_compare = parts[1].to_string();

        Ok(Self {
            operator: Operator::from_str(&operator).unwrap(),
            value_to_compare: U256::from_str(&value_to_compare)?,
        })
    }
}

impl std::fmt::Display for FunctionContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.operator, self.value_to_compare)
    }
}

impl FunctionContext {
    pub fn new(operator: Operator, value_to_compare: U256) -> Self {
        Self {
            operator,
            value_to_compare,
        }
    }
}

impl AggregationFunction {
    pub fn to_index(function_id: &Self) -> u8 {
        match function_id {
            AggregationFunction::AVG => 0,
            AggregationFunction::SUM => 1,
            AggregationFunction::MIN => 2,
            AggregationFunction::MAX => 3,
            AggregationFunction::COUNT => 4,
            AggregationFunction::MERKLE => 5,
            AggregationFunction::SLR => 6,
        }
    }

    pub fn from_index(index: u8) -> Result<Self> {
        match index {
            0 => Ok(AggregationFunction::AVG),
            1 => Ok(AggregationFunction::SUM),
            2 => Ok(AggregationFunction::MIN),
            3 => Ok(AggregationFunction::MAX),
            4 => Ok(AggregationFunction::COUNT),
            5 => Ok(AggregationFunction::MERKLE),
            6 => Ok(AggregationFunction::SLR),
            _ => bail!("Unknown aggregation function index"),
        }
    }

    pub fn operation(&self, values: &[U256], ctx: Option<FunctionContext>) -> Result<U256> {
        match self {
            // Aggregation functions for integer values
            AggregationFunction::AVG => integer::average(values),
            AggregationFunction::MAX => integer::find_max(values),
            AggregationFunction::MIN => integer::find_min(values),
            AggregationFunction::SUM => integer::sum(values),
            AggregationFunction::COUNT => {
                if let Some(ctx) = ctx {
                    integer::count(values, &ctx)
                } else {
                    bail!("Context not provided for COUNT")
                }
            }
            // Aggregation functions for string values
            AggregationFunction::MERKLE => todo!("Merkleize not implemented yet"),
            AggregationFunction::SLR => integer::simple_linear_regression(values),
        }
    }

    // Check if the function is pre-processable
    pub fn is_pre_processable(&self) -> bool {
        match self {
            AggregationFunction::AVG
            | AggregationFunction::SUM
            | AggregationFunction::MIN
            | AggregationFunction::MAX
            | AggregationFunction::COUNT => true,
            AggregationFunction::SLR | AggregationFunction::MERKLE => false,
        }
    }
}

// TODO: legacy parse.
// // Remove the "0x" prefix if exist, so that integer functions can parse integer values
// // In case of storage value, either if this is number or hex string type, all stored in hex string format.
// // So, we need to remove the "0x" prefix to parse the integer value if user target to use integer functions.
// // If the value is already in integer format, then it will be parsed as integer, which is decimal format.
// //
// // This also implies, even if the value is in hex string format, it will be parsed as integer, which is decimal format.
// // So for user it's importantant to know the value type and the function type.
// fn parse_int_value(values: &[String]) -> Result<Vec<U256>> {
//     let int_values: Vec<U256> = values
//         .iter()
//         .map(|hex_str| {
//             if hex_str.starts_with("0x") {
//                 let hex_value = hex_str.trim_start_matches("0x").to_string();
//                 U256::from_str_radix(&hex_value, 16).unwrap()
//             } else {
//                 U256::from_str_radix(hex_str, 10).unwrap()
//             }
//         })
//         .collect();

//     Ok(int_values)
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sum() {
        let sum_fn = AggregationFunction::SUM;

        // 4952100 ~ 4952100, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.nonce
        let values = vec![U256::from_str_radix("6776", 10).unwrap()];
        let result = sum_fn.operation(&values, None).unwrap();
        assert_eq!(result, U256::from(6776));

        // 4952100 ~ 4952103, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.nonce
        let values = vec![
            U256::from_str_radix("6776", 10).unwrap(),
            U256::from_str_radix("6776", 10).unwrap(),
            U256::from_str_radix("6776", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
        ];
        let result = sum_fn.operation(&values, None).unwrap();
        assert_eq!(result, U256::from(27105));

        // 5382810 ~ 5382810, storage.0x75CeC1db9dCeb703200EAa6595f66885C962B920.0x0000000000000000000000000000000000000000000000000000000000000002
        let values = vec![U256::from_str_radix("9184e72a000", 16).unwrap()];
        let result = sum_fn.operation(&values, None).unwrap();
        assert_eq!(result, U256::from_str_radix("10000000000000", 10).unwrap());

        // 5382810 ~ 5382813, storage.0x75CeC1db9dCeb703200EAa6595f66885C962B920.0x0000000000000000000000000000000000000000000000000000000000000002
        let values = vec![
            U256::from_str_radix("9184e72a000", 16).unwrap(),
            U256::from_str_radix("9184e72a000", 16).unwrap(),
            U256::from_str_radix("9184e72a000", 16).unwrap(),
            U256::from_str_radix("9184e72a000", 16).unwrap(),
        ];
        let result = sum_fn.operation(&values, None).unwrap();
        assert_eq!(result, U256::from_str_radix("40000000000000", 10).unwrap());

        // 4952100 ~ 4952103, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.balance
        let values = vec![
            U256::from_str_radix("41697298409483537348", 10).unwrap(),
            U256::from_str_radix("41697298409483537348", 10).unwrap(),
            U256::from_str_radix("41697298409483537348", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
        ];
        let result = sum_fn.operation(&values, None).unwrap();
        assert_eq!(
            result,
            U256::from_str_radix("166788991167020783608", 10).unwrap()
        );
    }

    #[test]
    fn test_avg() {
        let avg_fn = AggregationFunction::AVG;

        // 4952100 ~ 4952100, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.nonce
        let values = vec![U256::from_str_radix("6776", 10).unwrap()];
        let result = avg_fn.operation(&values, None).unwrap();
        assert_eq!(result, U256::from(6776));

        // 4952100 ~ 4952110, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.nonce
        let values = vec![
            U256::from_str_radix("6776", 10).unwrap(),
            U256::from_str_radix("6776", 10).unwrap(),
            U256::from_str_radix("6776", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
        ];
        let result = avg_fn.operation(&values, None).unwrap();
        assert_eq!(result, U256::from(6777));

        // 5382810 ~ 5382810, storage.0x75CeC1db9dCeb703200EAa6595f66885C962B920.0x0000000000000000000000000000000000000000000000000000000000000002
        let values = vec![U256::from_str_radix("9184e72a000", 16).unwrap()];
        let result = avg_fn.operation(&values, None).unwrap();
        assert_eq!(result, U256::from(10000000000000u64));

        // 5382810 ~ 5382813, storage.0x75CeC1db9dCeb703200EAa6595f66885C962B920.0x0000000000000000000000000000000000000000000000000000000000000002
        let values = vec![
            U256::from_str_radix("9184e72a000", 16).unwrap(),
            U256::from_str_radix("9184e72a000", 16).unwrap(),
            U256::from_str_radix("9184e72a000", 16).unwrap(),
            U256::from_str_radix("9184e72a000", 16).unwrap(),
        ];
        let result = avg_fn.operation(&values, None).unwrap();
        assert_eq!(result, U256::from(10000000000000u64));

        // 4952100 ~ 4952110, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.balance
        let values = vec![
            U256::from_str_radix("41697298409483537348", 10).unwrap(),
            U256::from_str_radix("41697298409483537348", 10).unwrap(),
            U256::from_str_radix("41697298409483537348", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
        ];
        let result = avg_fn.operation(&values, None).unwrap();
        assert_eq!(result, U256::from(41697151157910180414u128));
    }

    #[test]
    fn test_max() {
        let max_fn = AggregationFunction::MAX;

        // 4952100 ~ 4952100, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.nonce
        let values = vec![U256::from_str_radix("6776", 10).unwrap()];
        let result = max_fn.operation(&values, None).unwrap();
        assert_eq!(result, U256::from(6776));

        // 4952100 ~ 4952110, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.nonce
        let values = vec![
            U256::from_str_radix("6776", 10).unwrap(),
            U256::from_str_radix("6776", 10).unwrap(),
            U256::from_str_radix("6776", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
        ];
        let result = max_fn.operation(&values, None).unwrap();
        assert_eq!(result, U256::from(6777));

        // 5382810 ~ 5382810, storage.0x75CeC1db9dCeb703200EAa6595f66885C962B920.0x0000000000000000000000000000000000000000000000000000000000000002
        let values = vec![U256::from_str_radix("9184e72a000", 16).unwrap()];
        let result = max_fn.operation(&values, None).unwrap();
        assert_eq!(result, U256::from(10000000000000u64));

        // 5382810 ~ 5382813, storage.0x75CeC1db9dCeb703200EAa6595f66885C962B920.0x0000000000000000000000000000000000000000000000000000000000000002
        let values = vec![
            U256::from_str_radix("9184e72a000", 16).unwrap(),
            U256::from_str_radix("9184e72a000", 16).unwrap(),
            U256::from_str_radix("9184e72a000", 16).unwrap(),
            U256::from_str_radix("9184e72a000", 16).unwrap(),
        ];
        let result = max_fn.operation(&values, None).unwrap();
        assert_eq!(result, U256::from(10000000000000u64));

        // 4952100 ~ 4952110, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.balance
        let values = vec![
            U256::from_str_radix("41697298409483537348", 10).unwrap(),
            U256::from_str_radix("41697298409483537348", 10).unwrap(),
            U256::from_str_radix("41697298409483537348", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
        ];
        let result = max_fn.operation(&values, None).unwrap();
        assert_eq!(result, U256::from(41697298409483537348u128));
    }

    #[test]
    fn test_min() {
        let min_fn = AggregationFunction::MIN;

        // 4952100 ~ 4952100, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.nonce
        let values = vec![U256::from_str_radix("6776", 10).unwrap()];
        let result = min_fn.operation(&values, None).unwrap();
        assert_eq!(result, U256::from(6776));

        // 4952100 ~ 4952110, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.nonce
        let values = vec![
            U256::from_str_radix("6776", 10).unwrap(),
            U256::from_str_radix("6776", 10).unwrap(),
            U256::from_str_radix("6776", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
        ];
        let result = min_fn.operation(&values, None).unwrap();
        assert_eq!(result, U256::from(6776));

        // 5382810 ~ 5382810, storage.0x75CeC1db9dCeb703200EAa6595f66885C962B920.0x0000000000000000000000000000000000000000000000000000000000000002
        let values = vec![U256::from_str_radix("9184e72a000", 16).unwrap()];
        let result = min_fn.operation(&values, None).unwrap();
        assert_eq!(result, U256::from(10000000000000u64));

        // 5382810 ~ 5382813, storage.0x75CeC1db9dCeb703200EAa6595f66885C962B920.0x0000000000000000000000000000000000000000000000000000000000000002
        let values = vec![
            U256::from_str_radix("9184e72a000", 16).unwrap(),
            U256::from_str_radix("9184e72a000", 16).unwrap(),
            U256::from_str_radix("9184e72a000", 16).unwrap(),
            U256::from_str_radix("9184e72a000", 16).unwrap(),
        ];
        let result = min_fn.operation(&values, None).unwrap();
        assert_eq!(result, U256::from(10000000000000u64));

        // 4952100 ~ 4952110, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.balance
        let values = vec![
            U256::from_str_radix("41697298409483537348", 10).unwrap(),
            U256::from_str_radix("41697298409483537348", 10).unwrap(),
            U256::from_str_radix("41697298409483537348", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
        ];
        let result = min_fn.operation(&values, None).unwrap();
        assert_eq!(result, U256::from(41697095938570171564u128));
    }

    #[test]
    fn test_count() {
        let count = AggregationFunction::COUNT;

        // 4952100 ~ 4952100, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.nonce
        let values = vec![U256::from_str_radix("6776", 10).unwrap()];
        // logical_operator: 03 (>=)
        // value_to_compare: 0x0000000000000000000000000000000000000000000000000000000000000fff (4095)
        let result = count
            .operation(
                &values,
                Some(FunctionContext::new(
                    Operator::GreaterThanOrEqual,
                    U256::from(4095),
                )),
            )
            .unwrap();
        assert_eq!(result, U256::from(1));
        // logical_operator: 00 (=)
        // value_to_compare: 0x0000000000000000000000000000000000000000000000000000000000001A78 (6776)
        let result = count
            .operation(
                &values,
                Some(FunctionContext::new(Operator::Equal, U256::from(6776))),
            )
            .unwrap();
        assert_eq!(result, U256::from(1));

        // 4952100 ~ 4952110, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.nonce
        let values = vec![
            U256::from_str_radix("6776", 10).unwrap(),
            U256::from_str_radix("6776", 10).unwrap(),
            U256::from_str_radix("6776", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
            U256::from_str_radix("6777", 10).unwrap(),
        ];
        // logical_operator: 01 (!=)
        // value_to_compare: 0x0000000000000000000000000000000000000000000000000000000000001A78 (6776)
        let result = count
            .operation(
                &values,
                Some(FunctionContext::new(Operator::NotEqual, U256::from(6776))),
            )
            .unwrap();
        assert_eq!(result, U256::from(8));

        // logical_operator: 02 (>)
        // value_to_compare: 0x0000000000000000000000000000000000000000000000000000000000001A78 (6776)
        let result = count
            .operation(
                &values,
                Some(FunctionContext::new(
                    Operator::GreaterThan,
                    U256::from(6776),
                )),
            )
            .unwrap();
        assert_eq!(result, U256::from(8));

        // 5382810 ~ 5382810, storage.0x75CeC1db9dCeb703200EAa6595f66885C962B920.0x0000000000000000000000000000000000000000000000000000000000000002
        let values = vec![U256::from_str_radix("9184e72a000", 16).unwrap()];
        // logical_operator: 00 (=)
        // value_to_compare: 0x000000000000000000000000000000000000000000000000000009184e72a000 (10000000000000)
        let result = count
            .operation(
                &values,
                Some(FunctionContext::new(
                    Operator::Equal,
                    U256::from_str("10000000000000").unwrap(),
                )),
            )
            .unwrap();
        assert_eq!(result, U256::from(1));

        // 5382810 ~ 5382813, storage.0x75CeC1db9dCeb703200EAa6595f66885C962B920.0x0000000000000000000000000000000000000000000000000000000000000002
        // logical_operator: 05 (<=)
        // value_to_compare: 0x000000000000000000000000000000000000000000000000000009184e72a001 (10000000000001)
        let values = vec![
            U256::from_str_radix("9184e72a000", 16).unwrap(),
            U256::from_str_radix("9184e72a000", 16).unwrap(),
            U256::from_str_radix("9184e72a000", 16).unwrap(),
            U256::from_str_radix("9184e72a000", 16).unwrap(),
        ];
        let result = count
            .operation(
                &values,
                Some(FunctionContext::new(
                    Operator::LessThanOrEqual,
                    U256::from_str("10000000000001").unwrap(),
                )),
            )
            .unwrap();
        assert_eq!(result, U256::from(4));

        // 4952100 ~ 4952110, account.0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4.balance
        // logical_operator: 05 (<=)
        // value_to_compare: 0x00000000000000000000000000000000000000000000000242a9d7d5dfdbb4ac (41697095938570171564)
        let values = vec![
            U256::from_str_radix("41697298409483537348", 10).unwrap(),
            U256::from_str_radix("41697298409483537348", 10).unwrap(),
            U256::from_str_radix("41697298409483537348", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
            U256::from_str_radix("41697095938570171564", 10).unwrap(),
        ];
        let result = count
            .operation(
                &values,
                Some(FunctionContext::new(
                    Operator::LessThanOrEqual,
                    U256::from_str("41697095938570171564").unwrap(),
                )),
            )
            .unwrap();
        assert_eq!(result, U256::from(8));
    }
}
