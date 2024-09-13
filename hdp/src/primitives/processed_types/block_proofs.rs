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

pub fn mmr_with_header_vec_to_map(
    target: Vec<MMRWithHeader>,
) -> HashMap<MMRMeta, HashSet<ProcessedHeader>> {
    let mut map = HashMap::new();
    for target_item in target {
        map.entry(target_item.mmr_meta)
            .and_modify(|existing_headers: &mut HashSet<ProcessedHeader>| {
                existing_headers.extend(target_item.headers.iter().cloned());
            })
            .or_insert_with(|| target_item.headers.into_iter().collect());
    }
    map
}

impl MMRWithHeader {
    pub fn to_map(self) -> HashMap<MMRMeta, HashSet<ProcessedHeader>> {
        let mut map = HashMap::new();
        map.insert(self.mmr_meta, HashSet::from_iter(self.headers));
        map
    }

    pub fn extend(self, other: MMRWithHeader) -> Vec<MMRWithHeader> {
        let mut self_map = self.to_map();
        let other_map = other.to_map();
        for (mmr, headers) in other_map {
            self_map
                .entry(mmr)
                .and_modify(|existing_headers| {
                    existing_headers.extend(headers.iter().cloned());
                })
                .or_insert_with(|| headers.into_iter().collect());
        }
        convert_to_mmr_with_headers(self_map)
    }
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
