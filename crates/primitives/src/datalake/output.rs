use std::{fmt::Debug, str::FromStr};

use alloy_primitives::{
    hex::{self, FromHex},
    FixedBytes, B256,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct HeaderProof {
    pub leaf_idx: u64,
    pub mmr_path: Vec<String>,
}

/// HeaderProofFormatted is the formatted version of HeaderProof
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct HeaderProofFormatted {
    pub leaf_idx: u64,
    // mmr_path is encoded with poseidon
    pub mmr_path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct Header {
    pub rlp: String,
    pub proof: HeaderProof,
}

impl Header {
    pub fn to_cairo_format(&self) -> HeaderFormatted {
        println!("rlp: {:?}", &format!("0x{}", &self.rlp));
        let felts_unit = FieldElementVectorUnit::from_hex_str(&format!("0x{}", &self.rlp)).unwrap();
        let proof = self.proof.clone();
        HeaderFormatted {
            rlp: felts_unit.felts,
            rlp_bytes_len: felts_unit.bytes_len,
            proof: HeaderProofFormatted {
                leaf_idx: proof.leaf_idx,
                mmr_path: proof.mmr_path,
            },
        }
    }
}

/// HeaderFormatted is the formatted version of Header
#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct HeaderFormatted {
    #[serde_as(as = "Vec<UfeHex>")]
    pub rlp: Vec<FieldElement>,
    /// rlp_bytes_len is the byte( 8 bit ) length from rlp string
    pub rlp_bytes_len: u64,
    pub proof: HeaderProofFormatted,
}

#[serde_as]
#[derive(Serialize)]
pub struct FieldElementVectorUnit {
    #[serde_as(as = "Vec<UfeHex>")]
    pub felts: Vec<FieldElement>,
    pub bytes_len: u64,
}

impl FieldElementVectorUnit {
    pub fn from_hex_str(hex_str: &str) -> Result<Self> {
        // Convert hex string to bytes
        let bytes = hex::decode(hex_str).expect("Invalid hex input");
        let bytes_len = bytes.len() as u64;
        // Process bytes into 8-byte chunks and convert to little-endian u64, then to hex strings
        let felts = bytes
            .chunks(8)
            .map(|chunk| {
                let mut arr = [0u8; 8];
                let len = chunk.len();
                arr[..len].copy_from_slice(chunk);
                let le_int = u64::from_le_bytes(arr);
                FieldElement::from_dec_str(&le_int.to_string())
                    .expect("Invalid to convert FieldElement")
            })
            .collect();

        Ok(Self { felts, bytes_len })
    }
}

// pub fn combine_parts_into_big_endian_hex(uint256: &Uint256) -> String {
//     // Remove the "0x" prefix if present
//     let high = uint256.high.trim_start_matches("0x");
//     let low = uint256.low.trim_start_matches("0x");

//     // Ensure both parts are exactly 32 hex characters long
//     let high_padded = format!("{:0>32}", high);
//     let low_padded = format!("{:0>32}", low);

//     format!("0x{}{}", high_padded, low_padded)
// }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct Task {
    /// encoded computational task
    pub encoded_task: String,
    /// computational task commitment
    pub task_commitment: String,
    /// raw evaluation result of target compiled task
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compiled_result: Option<String>,
    /// results merkle tree's entry value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_commitment: Option<String>,
    pub task_proof: Vec<FixedBytes<32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_proof: Option<Vec<FixedBytes<32>>>,
    /// encoded datalake
    pub encoded_datalake: String,
    // ex. block sampled datalake / transaction datalake
    pub datalake_type: u8,
    // ex. "header", "account", "storage"
    pub property_type: u8,
}

impl Task {
    pub fn to_cairo_format(&self) -> TaskFormatted {
        let computational_task_felts =
            FieldElementVectorUnit::from_hex_str(&self.encoded_task).unwrap();
        let datalake_felts = FieldElementVectorUnit::from_hex_str(&self.encoded_datalake).unwrap();
        TaskFormatted {
            task_bytes_len: computational_task_felts.bytes_len,
            encoded_task: computational_task_felts.felts,
            datalake_bytes_len: datalake_felts.bytes_len,
            encoded_datalake: datalake_felts.felts,
            datalake_type: self.datalake_type,
            property_type: self.property_type,
        }
    }
}

#[serde_as]
#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
pub struct TaskFormatted {
    pub task_bytes_len: u64,
    #[serde_as(as = "Vec<UfeHex>")]
    pub encoded_task: Vec<FieldElement>,
    pub datalake_bytes_len: u64,
    #[serde_as(as = "Vec<UfeHex>")]
    pub encoded_datalake: Vec<FieldElement>,
    pub datalake_type: u8,
    pub property_type: u8,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct MPTProof {
    pub block_number: u64,
    pub proof: Vec<String>,
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct MPTProofFormatted {
    pub block_number: u64,
    /// proof_bytes_len is the byte( 8 bit ) length from each proof string
    pub proof_bytes_len: Vec<u64>,
    #[serde_as(as = "Vec<Vec<UfeHex>>")]
    pub proof: Vec<Vec<FieldElement>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct MMRMeta {
    pub id: u64,
    pub root: String,
    pub size: u64,
    // hex encoded
    pub peaks: Vec<String>,
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

    // #[test]
    // fn test_serde() {
    //     let target = Uint256::from_felts(
    //         felt!("0x1d693b26fded2cc4edb9acbf8133f52d"),
    //         felt!("0x2a32ca1db17188d788996d243adbda8d"),
    //     );
    //     let string = serde_json::to_string(&target).unwrap();
    //     assert_eq!(string, "{\"low\":\"0x2a32ca1db17188d788996d243adbda8d\",\"high\":\"0x1d693b26fded2cc4edb9acbf8133f52d\"}");

    //     let target = Uint256::from_felts(felt!("0x1"), felt!("0x2"));
    //     let string = serde_json::to_string(&target).unwrap();
    //     assert_eq!(string, "{\"low\":\"0x2\",\"high\":\"0x1\"}")
    // }
}
