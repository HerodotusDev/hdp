use serde::{Deserialize, Serialize};

use super::{
    account::ProcessedAccount, header::ProcessedHeader, mmr::MMRMeta, receipt::ProcessedReceipt,
    storage::ProcessedStorage, transaction::ProcessedTransaction,
};

/// Provider should fetch all the proofs and rlp values from given keys.

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessedBlockProofs {
    pub mmr_meta: MMRMeta,
    pub headers: Vec<ProcessedHeader>,
    pub accounts: Vec<ProcessedAccount>,
    pub storages: Vec<ProcessedStorage>,
    pub transactions: Vec<ProcessedTransaction>,
    pub transaction_receipts: Vec<ProcessedReceipt>,
}
