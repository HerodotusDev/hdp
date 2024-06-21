use serde::{Deserialize, Serialize};

use super::{
    account::ProcessedAccount, datalake_compute::ProcessedDatalakeCompute, header::ProcessedHeader,
    mmr::MMRMeta, receipt::ProcessedReceipt, storage::ProcessedStorage,
    transaction::ProcessedTransaction,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessedResult {
    // U256 type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results_root: Option<String>,
    // U256 type
    pub tasks_root: String,
    pub headers: Vec<ProcessedHeader>,
    pub mmr: MMRMeta,
    pub accounts: Vec<ProcessedAccount>,
    pub storages: Vec<ProcessedStorage>,
    pub transactions: Vec<ProcessedTransaction>,
    pub transaction_receipts: Vec<ProcessedReceipt>,
    pub tasks: Vec<ProcessedDatalakeCompute>,
}

impl ProcessedResult {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        results_root: Option<String>,
        tasks_root: String,
        headers: Vec<ProcessedHeader>,
        mmr: MMRMeta,
        accounts: Vec<ProcessedAccount>,
        storages: Vec<ProcessedStorage>,
        transactions: Vec<ProcessedTransaction>,
        transaction_receipts: Vec<ProcessedReceipt>,
        tasks: Vec<ProcessedDatalakeCompute>,
    ) -> Self {
        Self {
            results_root,
            tasks_root,
            headers,
            mmr,
            accounts,
            storages,
            transactions,
            transaction_receipts,
            tasks,
        }
    }

    pub fn update_results_root(&mut self, results_root: String) {
        self.results_root = Some(results_root);
    }
}
