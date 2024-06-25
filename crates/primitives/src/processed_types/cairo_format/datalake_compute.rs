use crate::processed_types::datalake_compute::ProcessedDatalakeCompute as BaseProcessedDatalakeCompute;

use super::{felt_vec_unit::FieldElementVectorUnit, traits::AsCairoFormat};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;

impl AsCairoFormat for BaseProcessedDatalakeCompute {
    type Output = ProcessedDatalakeCompute;

    fn as_cairo_format(&self) -> Self::Output {
        let computational_task_felts =
            FieldElementVectorUnit::from_bytes(&self.encoded_task).unwrap();
        let datalake_felts = FieldElementVectorUnit::from_bytes(&self.encoded_datalake).unwrap();
        ProcessedDatalakeCompute {
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
pub struct ProcessedDatalakeCompute {
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
        let datalake_computes: BaseProcessedDatalakeCompute =
            serde_json::from_str(&processed_string).unwrap();
        let datalake_computes_in_felts: ProcessedDatalakeCompute =
            datalake_computes.as_cairo_format();
        let string = serde_json::to_string_pretty(&datalake_computes_in_felts).unwrap();

        let json_file =
            fs::read_to_string("./fixtures/processed_in_felts/datalake_compute.json").unwrap();
        let expected: ProcessedDatalakeCompute = serde_json::from_str(&json_file).unwrap();
        let expected_string = serde_json::to_string_pretty(&expected).unwrap();

        assert_eq!(string, expected_string);
    }
}
