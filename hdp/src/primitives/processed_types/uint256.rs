//! This module contains the `Uint256` type, which is a 256-bit unsigned integer.
//! This is compatible with Cairo `uint256` type.

use alloy::primitives::{hex::FromHex, B256, U256};
use anyhow::Result;
use core::fmt::Display;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;
use std::str::FromStr;

use crate::primitives::utils::bytes_to_hex_string;

/// [`Uint256`] represents a 256-bit unsigned integer.
/// It is implemented as a struct with two [`FieldElement`] values: `high` and `low`.
/// Each [`FieldElement`] represents 128 bits of the 256-bit integer.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct Uint256 {
    #[serde_as(as = "UfeHex")]
    pub low: FieldElement, // Represents the least significant 128 bits
    #[serde_as(as = "UfeHex")]
    pub high: FieldElement, // Represents the most significant 128 bits
}

impl Default for Uint256 {
    /// Returns a `Uint256` with a value of zero.
    fn default() -> Self {
        Self::ZERO
    }
}

impl Uint256 {
    /// Constant representing zero as a `Uint256`.
    pub const ZERO: Self = Self {
        high: FieldElement::ZERO,
        low: FieldElement::ZERO,
    };

    /// Creates a `Uint256` from two byte slices representing the high and low parts.
    ///
    /// # Arguments
    /// * `high` - A byte slice representing the most significant 128 bits
    /// * `low` - A byte slice representing the least significant 128 bits
    ///
    /// # Returns
    /// A `Result` containing the new `Uint256` or an error if conversion fails.
    pub fn from_bytes_tuple(high: &[u8], low: &[u8]) -> Result<Self> {
        Ok(Self {
            high: FieldElement::from_byte_slice_be(high)?,
            low: FieldElement::from_byte_slice_be(low)?,
        })
    }

    /// Creates a `Uint256` from two hexadecimal strings representing the high and low parts.
    ///
    /// # Arguments
    /// * `high` - A string slice representing the most significant 128 bits in hex
    /// * `low` - A string slice representing the least significant 128 bits in hex
    ///
    /// # Returns
    /// A `Result` containing the new `Uint256` or an error if conversion fails.
    pub fn from_hex_tuple(high: &str, low: &str) -> Result<Self> {
        Ok(Self {
            high: FieldElement::from_hex_be(high)?,
            low: FieldElement::from_hex_be(low)?,
        })
    }

    /// Creates a `Uint256` from two [`FieldElement`]s representing the high and low parts.
    ///
    /// # Arguments
    /// * `high` - A `FieldElement` representing the most significant 128 bits
    /// * `low` - A `FieldElement` representing the least significant 128 bits
    ///
    /// # Returns
    /// A new `Uint256` instance.
    pub fn from_field_element_tuple(high: FieldElement, low: FieldElement) -> Self {
        Self { high, low }
    }

    /// Creates a `Uint256` from a little-endian hexadecimal string.
    ///
    /// # Arguments
    /// * `hex_str` - A string slice containing the hexadecimal representation
    ///
    /// # Returns
    /// A `Result` containing the new `Uint256` or an error if conversion fails.
    pub fn from_le_hex_str(hex_str: &str) -> Result<Self> {
        let clean_hex = hex_str.trim_start_matches("0x");
        let mut fix_hex: B256 = B256::from_hex(clean_hex)?;
        fix_hex.reverse();

        let high_part = fix_hex[..16].to_vec();
        let low_part = fix_hex[16..].to_vec();

        Ok(Self {
            high: FieldElement::from_hex_be(&bytes_to_hex_string(&high_part))?,
            low: FieldElement::from_hex_be(&bytes_to_hex_string(&low_part))?,
        })
    }

    /// Creates a `Uint256` from a big-endian hexadecimal string.
    ///
    /// # Arguments
    /// * `hex_str` - A string slice containing the hexadecimal representation
    ///
    /// # Returns
    /// A `Result` containing the new `Uint256` or an error if conversion fails.
    pub fn from_be_hex_str(hex_str: &str) -> Result<Self> {
        let clean_hex = hex_str.trim_start_matches("0x");
        let padded_hex = format!("{:0>64}", clean_hex);
        let (high_part, low_part) = padded_hex.split_at(32);
        Ok(Self {
            high: FieldElement::from_hex_be(&format!("0x{}", high_part))?,
            low: FieldElement::from_hex_be(&format!("0x{}", low_part))?,
        })
    }

    /// Creates a `Uint256` from a big-endian byte slice.
    ///
    /// # Arguments
    /// * `bytes` - A byte slice containing the 256-bit integer representation
    ///
    /// # Returns
    /// A `Result` containing the new `Uint256` or an error if conversion fails.
    pub fn from_be_bytes(bytes: &[u8]) -> Result<Self> {
        let high_part = bytes[..16].to_vec();
        let low_part = bytes[16..].to_vec();

        Ok(Self {
            high: FieldElement::from_hex_be(&bytes_to_hex_string(&high_part))?,
            low: FieldElement::from_hex_be(&bytes_to_hex_string(&low_part))?,
        })
    }
}

impl Display for Uint256 {
    /// Formats the `Uint256` as a hexadecimal string.
    /// The output is a 0x-prefixed, 64-character hexadecimal string.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Ensure both parts are exactly 32 hex characters long
        let high_padded = format!(
            "{:0>32}",
            bytes_to_hex_string(&self.high.to_bytes_be()[16..])
        )
        .trim_start_matches("0x")
        .to_string();
        let low_padded = format!(
            "{:0>32}",
            bytes_to_hex_string(&self.low.to_bytes_be()[16..])
        )
        .trim_start_matches("0x")
        .to_string();
        write!(f, "0x{}{}", high_padded, low_padded)
    }
}

impl From<Uint256> for B256 {
    /// Converts a `Uint256` to a `B256`.
    fn from(value: Uint256) -> B256 {
        B256::from_str(&value.to_string()).expect("Invalid value for B256")
    }
}

impl From<B256> for Uint256 {
    /// Converts a `B256` to a `Uint256`.
    fn from(value: B256) -> Self {
        Self::from_be_bytes(value.as_slice()).expect("Invalid value for Uint256")
    }
}

impl From<Uint256> for U256 {
    /// Converts a `Uint256` to a `U256`.
    fn from(value: Uint256) -> U256 {
        U256::from_str(&value.to_string()).expect("Invalid value for U256")
    }
}

impl From<U256> for Uint256 {
    /// Converts a `U256` to a `Uint256`.
    fn from(value: U256) -> Self {
        Self::from_be_bytes(&value.to_be_bytes_vec()).expect("Invalid value for Uint256")
    }
}

#[cfg(test)]
mod tests {

    use alloy::primitives::b256;
    use starknet::macros::felt;

    use super::*;

    #[test]
    fn test_combine_parts_into_big_endian_hex() {
        let result0: B256 = Uint256::from_field_element_tuple(
            felt!("0x988c19313bcbfb19fcc4da12e3adb46c"),
            felt!("0xf6fbdd08af91b1d8df80c6e755159f1"),
        )
        .into();
        assert_eq!(
            result0,
            b256!("988c19313bcbfb19fcc4da12e3adb46c0f6fbdd08af91b1d8df80c6e755159f1")
        );

        let result1: B256 = Uint256::from_field_element_tuple(
            felt!("0x988c19313bcbfb19fcc4da12e3adb46"),
            felt!("0xf6fbdd08af91b1d8df80c6e755159f1"),
        )
        .into();
        assert_eq!(
            result1,
            b256!("0988c19313bcbfb19fcc4da12e3adb460f6fbdd08af91b1d8df80c6e755159f1")
        );

        let result2: B256 = Uint256::from_field_element_tuple(
            felt!("0x988c19313bcbfb19fcc4da12e3adb4"),
            felt!("0xf6fbdd08af91b1d8df80c6e755159f1"),
        )
        .into();
        assert_eq!(
            result2,
            b256!("00988c19313bcbfb19fcc4da12e3adb40f6fbdd08af91b1d8df80c6e755159f1")
        );
    }

    #[test]
    fn test_split_big_endian_hex_into_parts() {
        let hex_str = "0x60870c80ce4e1d0c35e34f08b1648e8a4fdc7818eea7caedbd316c63a3863562";
        let result = Uint256::from_be_hex_str(hex_str).unwrap();
        assert_eq!(
            result,
            Uint256::from_field_element_tuple(
                felt!("0x60870c80ce4e1d0c35e34f08b1648e8a"),
                felt!("0x4fdc7818eea7caedbd316c63a3863562"),
            )
        );
        let result_b256: B256 = result.into();
        assert_eq!(result_b256, B256::from_str(hex_str).unwrap());
    }

    #[test]
    fn test_split_little_endian_hex_into_parts() {
        let hex_str = "0x8ddadb3a246d9988d78871b11dca322a2df53381bfacb9edc42cedfd263b691d";
        let result = Uint256::from_le_hex_str(hex_str).unwrap();
        assert_eq!(
            result,
            Uint256::from_field_element_tuple(
                felt!("0x1d693b26fded2cc4edb9acbf8133f52d"),
                felt!("0x2a32ca1db17188d788996d243adbda8d"),
            )
        );
    }

    #[test]
    fn test_uint256_serde() {
        let target = Uint256::from_field_element_tuple(
            felt!("0x1d693b26fded2cc4edb9acbf8133f52d"),
            felt!("0x2a32ca1db17188d788996d243adbda8d"),
        );
        let string = serde_json::to_string_pretty(&target).unwrap();
        let json_file = include_str!("../../../../fixtures/primitives/uint256.json");
        let expected = json_file.trim();
        assert_eq!(string, expected);
    }

    #[test]
    fn test_uint256_default() {
        let zero: B256 = Uint256::ZERO.into();

        assert_eq!(zero, B256::ZERO)
    }
}
