#[derive(Debug, Clone)]
pub struct FetchedAccountProof {
    pub block_number: u64,
    pub encoded_account: String,
    pub account_proof: Vec<String>,
}

/// Fetched storage and account proof and it's value
#[derive(Debug, Clone)]
pub struct FetchedStorageAccountProof {
    pub block_number: u64,
    pub encoded_account: String,
    pub account_proof: Vec<String>,
    pub storage_value: String,
    pub storage_proof: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FetchedTransactionProof {
    pub block_number: u64,
    pub tx_index: u64,
    pub encoded_transaction: String,
    pub transaction_proof: Vec<String>,
    pub tx_type: u8,
}

#[derive(Debug, Clone)]
pub struct FetchedTransactionReceiptProof {
    pub block_number: u64,
    pub tx_index: u64,
    pub encoded_receipt: String,
    pub receipt_proof: Vec<String>,
    pub tx_type: u8,
}
