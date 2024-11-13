use serde::{Deserialize, Serialize};
use starknet_crypto::Felt;

use crate::primitives::processed_types::header::ProcessedHeaderProof;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ProcessedHeader {
    pub fields: Vec<Felt>,
    pub proof: ProcessedHeaderProof,
}
