use super::{felt_vec_unit::FieldElementVectorUnit, traits::IntoFelts};
use alloy_primitives::B256;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ProcessedDatalakeCompute {
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
    pub task_proof: Vec<B256>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_proof: Option<Vec<B256>>,
    /// encoded datalake
    pub encoded_datalake: String,
    // ex. block sampled datalake / transaction datalake
    pub datalake_type: u8,
    // ex. "header", "account", "storage"
    pub property_type: u8,
}

impl IntoFelts for ProcessedDatalakeCompute {
    type Output = ProcessedDatalakeComputeInFelts;

    fn to_felts(&self) -> Self::Output {
        let computational_task_felts =
            FieldElementVectorUnit::from_hex_str(&self.encoded_task).unwrap();
        let datalake_felts = FieldElementVectorUnit::from_hex_str(&self.encoded_datalake).unwrap();
        ProcessedDatalakeComputeInFelts {
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
pub struct ProcessedDatalakeComputeInFelts {
    pub task_bytes_len: u64,
    #[serde_as(as = "Vec<UfeHex>")]
    pub encoded_task: Vec<FieldElement>,
    pub datalake_bytes_len: u64,
    #[serde_as(as = "Vec<UfeHex>")]
    pub encoded_datalake: Vec<FieldElement>,
    pub datalake_type: u8,
    pub property_type: u8,
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_datalake_compute_to_serde() {
        let processed_string =
            fs::read_to_string("fixtures/processed/datalake_compute.json").unwrap();
        let datalake_computes: ProcessedDatalakeCompute =
            serde_json::from_str(&processed_string).unwrap();
        let datalake_computes_in_felts: ProcessedDatalakeComputeInFelts =
            datalake_computes.to_felts();
        let string = serde_json::to_string_pretty(&datalake_computes_in_felts).unwrap();

        let json_file =
            fs::read_to_string("./fixtures/processed_in_felts/datalake_compute.json").unwrap();
        let expected: ProcessedDatalakeComputeInFelts = serde_json::from_str(&json_file).unwrap();
        let expected_string = serde_json::to_string_pretty(&expected).unwrap();

        assert_eq!(string, expected_string);
    }
}
