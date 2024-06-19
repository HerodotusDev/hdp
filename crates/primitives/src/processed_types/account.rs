//! Processed account type
//! This contains the processed account type and its conversion to cairo format.

use alloy_primitives::keccak256;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;

use super::{
    felt_vec_unit::FieldElementVectorUnit,
    mpt::{ProcessedMPTProof, ProcessedMPTProofInFelts},
    traits::IntoFelts,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ProcessedAccount {
    pub address: String,
    pub account_key: String,
    pub proofs: Vec<ProcessedMPTProof>,
}

impl ProcessedAccount {
    pub fn new(address: String, proofs: Vec<ProcessedMPTProof>) -> Self {
        let account_key = keccak256(&address).to_string();
        ProcessedAccount {
            address,
            account_key,
            proofs,
        }
    }
}

impl IntoFelts for ProcessedAccount {
    type Output = ProcessedAccountInFelts;

    fn to_felts(&self) -> Self::Output {
        let address_chunk_result = FieldElementVectorUnit::from_hex_str(&self.address).unwrap();
        let account_key = &self.account_key;
        let proofs = self
            .proofs
            .iter()
            .map(|proof| proof.to_cairo_format())
            .collect();
        ProcessedAccountInFelts {
            address: address_chunk_result.felts,
            account_key: account_key.into(),
            proofs,
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ProcessedAccountInFelts {
    #[serde_as(as = "Vec<UfeHex>")]
    pub address: Vec<FieldElement>,
    pub account_key: String,
    pub proofs: Vec<ProcessedMPTProofInFelts>,
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_account_to_serde() {
        let processed_string = fs::read_to_string("fixtures/processed/account.json").unwrap();
        let accounts: ProcessedAccount = serde_json::from_str(&processed_string).unwrap();
        let accounts_in_felts: ProcessedAccountInFelts = accounts.to_felts();
        let string = serde_json::to_string_pretty(&accounts_in_felts).unwrap();

        let json_file = fs::read_to_string("./fixtures/processed_in_felts/account.json").unwrap();
        let expected: ProcessedAccountInFelts = serde_json::from_str(&json_file).unwrap();
        let expected_string = serde_json::to_string_pretty(&expected).unwrap();

        assert_eq!(string, expected_string);
    }
}
