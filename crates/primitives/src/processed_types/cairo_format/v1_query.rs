use serde::{Deserialize, Serialize};

use crate::processed_types::{
    mmr::MMRMeta, uint256::Uint256, v1_query::ProcessedResult as BaseProcessedResult,
};

use super::{
    AsCairoFormat, ProcessedAccount, ProcessedDatalakeCompute, ProcessedHeader, ProcessedReceipt,
    ProcessedStorage, ProcessedTransaction,
};

impl AsCairoFormat for BaseProcessedResult {
    type Output = ProcessedResult;

    fn as_cairo_format(&self) -> ProcessedResult {
        let headers = self
            .headers
            .iter()
            .map(|header| header.as_cairo_format())
            .collect();
        let accounts = self
            .accounts
            .iter()
            .map(|account| account.as_cairo_format())
            .collect();
        let storages = self
            .storages
            .iter()
            .map(|storage| storage.as_cairo_format())
            .collect();
        let transactions = self
            .transactions
            .iter()
            .map(|transaction| transaction.as_cairo_format())
            .collect();
        let transaction_receipts = self
            .transaction_receipts
            .iter()
            .map(|receipt| receipt.as_cairo_format())
            .collect();
        let tasks = self
            .tasks
            .iter()
            .map(|task| task.as_cairo_format())
            .collect();
        let results_root = self
            .results_root
            .as_ref()
            .map(|root| Uint256::from_be_hex_str(root).unwrap());

        ProcessedResult {
            results_root,
            tasks_root: Uint256::from_be_hex_str(&self.tasks_root).unwrap(),
            headers,
            mmr: self.mmr.clone(),
            accounts,
            storages,
            transactions,
            transaction_receipts,
            tasks,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessedResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results_root: Option<Uint256>,
    pub tasks_root: Uint256,
    pub headers: Vec<ProcessedHeader>,
    pub mmr: MMRMeta,
    accounts: Vec<ProcessedAccount>,
    storages: Vec<ProcessedStorage>,
    transactions: Vec<ProcessedTransaction>,
    transaction_receipts: Vec<ProcessedReceipt>,
    pub tasks: Vec<ProcessedDatalakeCompute>,
}
