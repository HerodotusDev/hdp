use alloy::primitives::Bytes;

#[derive(Debug, Clone)]
pub struct FetchedTransactionProof {
    pub block_number: u64,
    pub tx_index: u64,
    pub encoded_transaction: Bytes,
    pub transaction_proof: Vec<Bytes>,
    pub tx_type: u8,
}

#[derive(Debug, Clone)]
pub struct FetchedTransactionReceiptProof {
    pub block_number: u64,
    pub tx_index: u64,
    pub encoded_receipt: Bytes,
    pub receipt_proof: Vec<Bytes>,
    pub tx_type: u8,
}
