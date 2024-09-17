//! Processed account type
//! This contains the processed account type and its conversion to cairo format.

use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_types_core::felt::Felt;

use crate::primitives::processed_types::account::ProcessedAccount as BaseProcessedAccount;

use super::{felt_vec_unit::FieldElementVectorUnit, mpt::ProcessedMPTProof, AsCairoFormat};

impl AsCairoFormat for BaseProcessedAccount {
    type Output = ProcessedAccount;

    fn as_cairo_format(&self) -> Self::Output {
        let address_chunk_result = FieldElementVectorUnit::from_bytes(self.address.as_ref());
        let account_key = &self.account_key;
        let proofs = self
            .proofs
            .iter()
            .map(|proof| proof.as_cairo_format())
            .collect();
        ProcessedAccount {
            address: address_chunk_result.felts,
            account_key: account_key.into(),
            proofs,
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ProcessedAccount {
    #[serde_as(as = "Vec<UfeHex>")]
    pub address: Vec<Felt>,
    pub account_key: String,
    pub proofs: Vec<ProcessedMPTProof>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_to_serde() {
        let processed_string =
            include_str!("../../../../../fixtures/primitives/processed/account.json");
        let accounts: BaseProcessedAccount = serde_json::from_str(processed_string).unwrap();
        let accounts_in_felts: ProcessedAccount = accounts.as_cairo_format();
        let string = serde_json::to_string_pretty(&accounts_in_felts).unwrap();

        let json_file =
            include_str!("../../../../../fixtures/primitives/processed_in_felts/account.json");
        let expected: ProcessedAccount = serde_json::from_str(json_file).unwrap();
        let expected_string = serde_json::to_string_pretty(&expected).unwrap();

        assert_eq!(string, expected_string);
    }
}
