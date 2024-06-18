use anyhow::Result;
use hdp_primitives::datalake::{
    block_sampled::output::{Account, AccountFormatted, Storage, StorageFormatted},
    output::{Header, HeaderFormatted, MMRMeta, Task, TaskFormatted, Uint256},
    transactions::output::{
        Transaction, TransactionFormatted, TransactionReceipt, TransactionReceiptFormatted,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessedResult {
    // U256 type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub results_root: Option<String>,
    // U256 type
    pub tasks_root: String,
    pub headers: Vec<Header>,
    pub mmr: MMRMeta,
    pub accounts: Vec<Account>,
    pub storages: Vec<Storage>,
    pub transactions: Vec<Transaction>,
    pub transaction_receipts: Vec<TransactionReceipt>,
    pub tasks: Vec<Task>,
}

impl ProcessedResult {
    pub fn to_cairo_format(&self) -> ProcessedResultFormatted {
        let headers = self
            .headers
            .iter()
            .map(|header| header.to_cairo_format())
            .collect();
        let accounts = self
            .accounts
            .iter()
            .map(|account| account.to_cairo_format())
            .collect();
        let storages = self
            .storages
            .iter()
            .map(|storage| storage.to_cairo_format())
            .collect();
        let transactions = self
            .transactions
            .iter()
            .map(|transaction| transaction.to_cairo_format())
            .collect();
        let transaction_receipts = self
            .transaction_receipts
            .iter()
            .map(|receipt| receipt.to_cairo_format())
            .collect();
        let tasks = self
            .tasks
            .iter()
            .map(|task| task.to_cairo_format())
            .collect();
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
    pub headers: Vec<HeaderFormatted>,
    pub mmr: MMRMeta,
    accounts: Vec<AccountFormatted>,
    storages: Vec<StorageFormatted>,
    transactions: Vec<TransactionFormatted>,
    transaction_receipts: Vec<TransactionReceiptFormatted>,
    pub tasks: Vec<TaskFormatted>,
}
