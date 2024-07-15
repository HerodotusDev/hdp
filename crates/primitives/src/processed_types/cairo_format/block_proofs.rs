use serde::Serialize;

use crate::processed_types::{
    block_proofs::ProcessedBlockProofs as BaseProcessedBlockProofs, mmr::MMRMeta,
};

use super::{
    AsCairoFormat, ProcessedAccount, ProcessedHeader, ProcessedReceipt, ProcessedStorage,
    ProcessedTransaction,
};

impl AsCairoFormat for BaseProcessedBlockProofs {
    type Output = ProcessedBlockProofs;

    fn as_cairo_format(&self) -> Self::Output {
        ProcessedBlockProofs {
            mmr_metas: self.mmr_metas.clone(),
            headers: self
                .headers
                .iter()
                .map(|header| header.as_cairo_format())
                .collect(),
            accounts: self
                .accounts
                .iter()
                .map(|account| account.as_cairo_format())
                .collect(),
            storages: self
                .storages
                .iter()
                .map(|storage| storage.as_cairo_format())
                .collect(),
            transactions: self
                .transactions
                .iter()
                .map(|transaction| transaction.as_cairo_format())
                .collect(),
            transaction_receipts: self
                .transaction_receipts
                .iter()
                .map(|receipt| receipt.as_cairo_format())
                .collect(),
        }
    }
}

#[derive(Serialize)]
pub struct ProcessedBlockProofs {
    pub mmr_metas: Vec<MMRMeta>,
    pub headers: Vec<ProcessedHeader>,
    pub accounts: Vec<ProcessedAccount>,
    pub storages: Vec<ProcessedStorage>,
    pub transactions: Vec<ProcessedTransaction>,
    pub transaction_receipts: Vec<ProcessedReceipt>,
}
