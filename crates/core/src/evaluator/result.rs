use anyhow::Result;

use hdp_primitives::processed_types::{
    account::{ProcessedAccount, ProcessedAccountInFelts},
    datalake_compute::{ProcessedDatalakeCompute, ProcessedDatalakeComputeInFelts},
    header::{ProcessedHeader, ProcessedHeaderInFelts},
    mmr::MMRMeta,
    receipt::{ProcessedReceipt, ProcessedReceiptInFelts},
    storage::{ProcessedStorage, ProcessedStorageInFelts},
    traits::IntoFelts,
    transaction::{ProcessedTransaction, ProcessedTransactionInFelts},
    uint256::Uint256,
};
use serde::{Deserialize, Serialize};

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
    pub fn to_cairo_format(&self) -> ProcessedResultFormatted {
        let headers = self
            .headers
            .iter()
            .map(|header| header.to_felts())
            .collect();
        let accounts = self
            .accounts
            .iter()
            .map(|account| account.to_felts())
            .collect();
        let storages = self
            .storages
            .iter()
            .map(|storage| storage.to_felts())
            .collect();
        let transactions = self
            .transactions
            .iter()
            .map(|transaction| transaction.to_felts())
            .collect();
        let transaction_receipts = self
            .transaction_receipts
            .iter()
            .map(|receipt| receipt.to_felts())
            .collect();
        let tasks = self.tasks.iter().map(|task| task.to_felts()).collect();
        let results_root = self
            .results_root
            .as_ref()
            .map(|root| Uint256::from_be_hex_str(root).unwrap());

        ProcessedResultFormatted {
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

    pub fn save_to_file(&self, file_path: &str, is_cairo_format: bool) -> Result<()> {
        let json = if is_cairo_format {
            self.to_cairo_formatted_json()?
        } else {
            self.to_general_json()?
        };
        std::fs::write(file_path, json)?;
        Ok(())
    }

    fn to_general_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self)?)
    }

    fn to_cairo_formatted_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self.to_cairo_format())?)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessedResultFormatted {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results_root: Option<Uint256>,
    pub tasks_root: Uint256,
    pub headers: Vec<ProcessedHeaderInFelts>,
    pub mmr: MMRMeta,
    accounts: Vec<ProcessedAccountInFelts>,
    storages: Vec<ProcessedStorageInFelts>,
    transactions: Vec<ProcessedTransactionInFelts>,
    transaction_receipts: Vec<ProcessedReceiptInFelts>,
    pub tasks: Vec<ProcessedDatalakeComputeInFelts>,
}
