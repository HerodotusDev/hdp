use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_types_core::felt::Felt;

use crate::primitives::processed_types::header::{
    ProcessedHeader as BaseProcessedHeader, ProcessedHeaderProof as BasedProcessedHeaderProof,
};

use super::{felt_vec_unit::FieldElementVectorUnit, traits::AsCairoFormat};

impl AsCairoFormat for BaseProcessedHeader {
    type Output = ProcessedHeader;

    fn as_cairo_format(&self) -> Self::Output {
        let felts_unit = FieldElementVectorUnit::from_bytes(&self.rlp);
        let proof = self.proof.clone();
        ProcessedHeader {
            rlp: felts_unit.felts,
            rlp_bytes_len: felts_unit.bytes_len,
            proof: BasedProcessedHeaderProof {
                leaf_idx: proof.leaf_idx,
                mmr_path: proof.mmr_path,
            },
        }
    }
}

/// HeaderFormatted is the formatted version of Header
#[serde_as]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ProcessedHeader {
    #[serde_as(as = "Vec<UfeHex>")]
    pub rlp: Vec<Felt>,
    /// rlp_bytes_len is the byte( 8 bit ) length from rlp string
    pub rlp_bytes_len: u64,
    pub proof: BasedProcessedHeaderProof,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_serde() {
        let processed_string =
            include_str!("../../../../../fixtures/primitives/processed/header.json");
        let headers: BaseProcessedHeader = serde_json::from_str(processed_string).unwrap();
        let headers_in_felts: ProcessedHeader = headers.as_cairo_format();
        let string = serde_json::to_string_pretty(&headers_in_felts).unwrap();

        let json_file =
            include_str!("../../../../../fixtures/primitives/processed_in_felts/header.json");
        let expected: ProcessedHeader = serde_json::from_str(json_file).unwrap();
        let expected_string = serde_json::to_string_pretty(&expected).unwrap();

        assert_eq!(string, expected_string);
    }
}
