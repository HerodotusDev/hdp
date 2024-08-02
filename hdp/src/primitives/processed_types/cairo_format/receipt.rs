//! This module defines the `ProcessedReceipt` struct and its corresponding `ProcessedReceiptInFelts` struct.

use super::{felt_vec_unit::FieldElementVectorUnit, traits::AsCairoFormat};
use crate::primitives::processed_types::receipt::ProcessedReceipt as BaseProcessedReceipt;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;

impl AsCairoFormat for BaseProcessedReceipt {
    type Output = ProcessedReceipt;

    fn as_cairo_format(&self) -> Self::Output {
        let key = self.key.clone();
        let proof_felts: Vec<FieldElementVectorUnit> = self
            .proof
            .iter()
            .map(|proof| FieldElementVectorUnit::from_bytes(proof).unwrap())
            .collect();

        let proof_bytes_len = proof_felts.iter().map(|f| f.bytes_len).collect();
        let proof_result: Vec<Vec<FieldElement>> =
            proof_felts.iter().map(|f| f.felts.clone()).collect();
        ProcessedReceipt {
            key,
            block_number: self.block_number,
            proof_bytes_len,
            proof: proof_result,
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
#[serde(rename = "transaction_receipts")]
pub struct ProcessedReceipt {
    pub key: String,
    pub block_number: u64,
    /// proof_bytes_len is the byte( 8 bit ) length from each proof string
    pub proof_bytes_len: Vec<u64>,
    #[serde_as(as = "Vec<Vec<UfeHex>>")]
    pub proof: Vec<Vec<FieldElement>>,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_receipt_serde() {
        let processed_string =
            include_str!("../../../../../fixtures/primitives/processed/receipt.json");
        let receipts: BaseProcessedReceipt = serde_json::from_str(processed_string).unwrap();
        let receipts_in_felts: ProcessedReceipt = receipts.as_cairo_format();
        let string = serde_json::to_string_pretty(&receipts_in_felts).unwrap();

        let json_file =
            include_str!("../../../../../fixtures/primitives/processed_in_felts/receipt.json");
        let expected: ProcessedReceipt = serde_json::from_str(json_file).unwrap();
        let expected_string = serde_json::to_string_pretty(&expected).unwrap();

        assert_eq!(string, expected_string);
    }
}
