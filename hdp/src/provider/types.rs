//! Types for the provider crate.
//! For the `FetchedTransactionProof` and `FetchedTransactionReceiptProof` types.
//!
//! We need this type to bind encoded transaction and receipts to the block number and proofs.

use std::collections::HashSet;

use crate::primitives::processed_types::{
    account::ProcessedAccount, header::ProcessedHeader, mmr::MMRMeta, receipt::ProcessedReceipt,
    storage::ProcessedStorage, transaction::ProcessedTransaction,
};

use alloy::primitives::U256;
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

pub struct FetchedDatalake {
    /// Targeted datalake's compiled results
    pub values: Vec<U256>,
    /// Headers related to the datalake
    pub headers: HashSet<ProcessedHeader>,
    /// Accounts related to the datalake
    pub accounts: HashSet<ProcessedAccount>,
    /// Storages related to the datalake
    pub storages: HashSet<ProcessedStorage>,
    /// Transactions related to the datalake
    pub transactions: HashSet<ProcessedTransaction>,
    /// Transaction receipts related to the datalake
    pub transaction_receipts: HashSet<ProcessedReceipt>,
    /// MMR meta data related to the headers
    pub mmr_metas: HashSet<MMRMeta>,
}
