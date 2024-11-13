use serde::{Deserialize, Serialize};

use crate::primitives::processed_types::{
    block_proofs::ProcessedBlockProofs as BaseProcessedBlockProofs, mmr::MMRMeta,
};

use super::{
    AsCairoFormat, ProcessedAccount, ProcessedHeader, ProcessedReceipt, ProcessedStorage,
    ProcessedTransaction,
};

impl AsCairoFormat for BaseProcessedBlockProofs {
    type Output = ProcessedBlockProofs;

    fn as_cairo_format(&self) -> Self::Output {
        let evm_proof = self.clone().get_evm_proofs().unwrap();
        ProcessedBlockProofs {
            chain_id: evm_proof.chain_id,
            mmr_with_headers: evm_proof
                .mmr_with_headers
                .iter()
                .map(|mmr_with_header| MMRWithHeader {
                    mmr_meta: mmr_with_header.mmr_meta.clone(),
                    headers: mmr_with_header
                        .headers
                        .iter()
                        .map(|header| header.as_cairo_format())
                        .collect(),
                })
                .collect(),
            accounts: evm_proof
                .accounts
                .iter()
                .map(|account| account.as_cairo_format())
                .collect(),
            storages: evm_proof
                .storages
                .iter()
                .map(|storage| storage.as_cairo_format())
                .collect(),
            transactions: evm_proof
                .transactions
                .iter()
                .map(|transaction| transaction.as_cairo_format())
                .collect(),
            transaction_receipts: evm_proof
                .transaction_receipts
                .iter()
                .map(|receipt| receipt.as_cairo_format())
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ProcessedBlockProofs {
    pub chain_id: u128,
    pub mmr_with_headers: Vec<MMRWithHeader>,
    pub accounts: Vec<ProcessedAccount>,
    pub storages: Vec<ProcessedStorage>,
    pub transactions: Vec<ProcessedTransaction>,
    pub transaction_receipts: Vec<ProcessedReceipt>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MMRWithHeader {
    pub mmr_meta: MMRMeta,
    pub headers: Vec<ProcessedHeader>,
}
