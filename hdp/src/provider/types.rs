//! Types for the provider crate.
//! For the `FetchedTransactionProof` and `FetchedTransactionReceiptProof` types.
//!
//! We need this type to bind encoded transaction and receipts to the block number and proofs.

use alloy::{
    consensus::TxType,
    primitives::{BlockNumber, Bytes, TxIndex},
};

#[derive(Debug, Clone)]
pub struct FetchedTransactionProof {
    pub block_number: BlockNumber,
    pub tx_index: TxIndex,
    pub encoded_transaction: Vec<u8>,
    pub transaction_proof: Vec<Bytes>,
    pub tx_type: TxType,
}

impl FetchedTransactionProof {
    pub fn new(
        block_number: BlockNumber,
        tx_index: TxIndex,
        encoded_transaction: Vec<u8>,
        transaction_proof: Vec<Bytes>,
        tx_type: TxType,
    ) -> Self {
        Self {
            block_number,
            tx_index,
            encoded_transaction,
            transaction_proof,
            tx_type,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FetchedTransactionReceiptProof {
    pub block_number: BlockNumber,
    pub tx_index: TxIndex,
    pub encoded_receipt: Vec<u8>,
    pub receipt_proof: Vec<Bytes>,
    pub tx_type: TxType,
}

impl FetchedTransactionReceiptProof {
    pub fn new(
        block_number: BlockNumber,
        tx_index: TxIndex,
        encoded_receipt: Vec<u8>,
        receipt_proof: Vec<Bytes>,
        tx_type: TxType,
    ) -> Self {
        Self {
            block_number,
            tx_index,
            encoded_receipt,
            receipt_proof,
            tx_type,
        }
    }
}
