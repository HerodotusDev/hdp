use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use super::{
    account::ProcessedAccount, header::ProcessedHeader, mmr::MMRMeta, receipt::ProcessedReceipt,
    storage::ProcessedStorage, transaction::ProcessedTransaction,
};

/// Provider should fetch all the proofs and rlp values from given keys.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct ProcessedBlockProofs {
    pub chain_id: u128,
    pub mmr_with_headers: Vec<MMRWithHeader>,
    pub accounts: Vec<ProcessedAccount>,
    pub storages: Vec<ProcessedStorage>,
    pub transactions: Vec<ProcessedTransaction>,
    pub transaction_receipts: Vec<ProcessedReceipt>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct MMRWithHeader {
    pub mmr_meta: MMRMeta,
    pub headers: Vec<ProcessedHeader>,
}

pub fn convert_to_mmr_with_headers(
    map: HashMap<MMRMeta, HashSet<ProcessedHeader>>,
) -> Vec<MMRWithHeader> {
    map.into_iter()
        .map(|(mmr_meta, headers)| MMRWithHeader {
            mmr_meta,
            headers: headers.into_iter().collect(),
        })
        .collect()
}

pub fn convert_to_mmr_meta_set(
    mmr_with_headers: Vec<MMRWithHeader>,
) -> HashMap<MMRMeta, HashSet<ProcessedHeader>> {
    mmr_with_headers
        .into_iter()
        .map(|mmr_with_header| {
            (
                mmr_with_header.mmr_meta,
                mmr_with_header.headers.into_iter().collect::<HashSet<_>>(),
            )
        })
        .collect()
}
