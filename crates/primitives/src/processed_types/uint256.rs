//! This module contains the `Uint256` type, which is a 256-bit unsigned integer.
//! This is compatible with Cairo `uint256` type.

use alloy::primitives::{hex::FromHex, B256};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;
use std::str::FromStr;

use crate::utils::bytes_to_hex_string;

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct Uint256 {
    #[serde_as(as = "UfeHex")]
    pub low: FieldElement,
    #[serde_as(as = "UfeHex")]
    pub high: FieldElement,
}

impl Uint256 {
    pub fn from_strs(high: &str, low: &str) -> Result<Self> {
        Ok(Self {
            high: FieldElement::from_hex_be(high)?,
            low: FieldElement::from_hex_be(low)?,
        })
    }

    pub fn from_felts(high: FieldElement, low: FieldElement) -> Self {
        Self { high, low }
    }

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

    pub fn from_be_hex_str(hex_str: &str) -> Result<Self> {
        let clean_hex = hex_str.trim_start_matches("0x");
        let padded_hex = format!("{:0>64}", clean_hex);
        let (high_part, low_part) = padded_hex.split_at(32);
        Ok(Self {
            high: FieldElement::from_hex_be(&format!("0x{}", high_part))?,
            low: FieldElement::from_hex_be(&format!("0x{}", low_part))?,
        })
    }

    /// combine_parts_into_big_endian_hex
    pub fn to_combined_string(&self) -> B256 {
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

        B256::from_str(&format!("0x{}{}", high_padded, low_padded)).unwrap()
    }
}

#[cfg(test)]
mod tests {

    use starknet::macros::felt;

    use super::*;

    #[test]
    fn test_combine_parts_into_big_endian_hex() {
        let uint256 = Uint256::from_felts(
            FieldElement::from_hex_be("0x988c19313bcbfb19fcc4da12e3adb46c").unwrap(),
            FieldElement::from_hex_be("0xf6fbdd08af91b1d8df80c6e755159f1").unwrap(),
        );
        assert_eq!(
            uint256.to_combined_string(),
            B256::from_str("0x988c19313bcbfb19fcc4da12e3adb46c0f6fbdd08af91b1d8df80c6e755159f1")
                .unwrap()
        );

        let uint256 = Uint256::from_felts(
            felt!("0x988c19313bcbfb19fcc4da12e3adb46"),
            felt!("0xf6fbdd08af91b1d8df80c6e755159f1"),
        );
        assert_eq!(
            uint256.to_combined_string(),
            B256::from_str("0x0988c19313bcbfb19fcc4da12e3adb460f6fbdd08af91b1d8df80c6e755159f1")
                .unwrap()
        );

        let uint256 = Uint256::from_felts(
            felt!("0x988c19313bcbfb19fcc4da12e3adb4"),
            felt!("0xf6fbdd08af91b1d8df80c6e755159f1"),
        );
        assert_eq!(
            uint256.to_combined_string(),
            B256::from_str("0x00988c19313bcbfb19fcc4da12e3adb40f6fbdd08af91b1d8df80c6e755159f1")
                .unwrap()
        );
    }

    #[test]
    fn test_split_big_endian_hex_into_parts() {
        let hex_str = "0x60870c80ce4e1d0c35e34f08b1648e8a4fdc7818eea7caedbd316c63a3863562";
        let result = Uint256::from_be_hex_str(hex_str).unwrap();
        assert_eq!(
            result,
            Uint256::from_felts(
                felt!("0x60870c80ce4e1d0c35e34f08b1648e8a"),
                felt!("0x4fdc7818eea7caedbd316c63a3863562"),
            )
        );
        assert_eq!(result.to_combined_string().to_string(), hex_str);
    }

    #[test]
    fn test_split_little_endian_hex_into_parts() {
        let hex_str = "0x8ddadb3a246d9988d78871b11dca322a2df53381bfacb9edc42cedfd263b691d";
        let result = Uint256::from_le_hex_str(hex_str).unwrap();
        assert_eq!(
            result,
            Uint256::from_felts(
                felt!("0x1d693b26fded2cc4edb9acbf8133f52d"),
                felt!("0x2a32ca1db17188d788996d243adbda8d"),
            )
        );
    }

    #[test]
    fn test_uint256_serde() {
        let target = Uint256::from_felts(
            felt!("0x1d693b26fded2cc4edb9acbf8133f52d"),
            felt!("0x2a32ca1db17188d788996d243adbda8d"),
        );
        let string = serde_json::to_string_pretty(&target).unwrap();
        let json_file = std::fs::read_to_string("./fixtures/uint256.json").unwrap();
        let expected = json_file.trim();
        assert_eq!(string, expected);
    }
}
