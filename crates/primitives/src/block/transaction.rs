use std::str::FromStr;

use anyhow::{bail, Result};

#[derive(Debug, PartialEq)]
pub enum TransactionsCollection {
    TransactionsBySender,
    TranasactionReceiptsBySender,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionDatalakeField {
    // ===== Transaction fields =====
    ChainId,
    Nonce,
    // Only for legacy transactions
    GasPrice,
    GasLimit,
    // EIP-1559 transactions and EIP-4844 transactions
    MaxFeePerGas,
    // EIP-1559 transactions and EIP-4844 transactions
    MaxPriorityFeePerGas,
    To,
    Value,
    // Not for legacy transactions
    AccessList,
    // Only for EIP-4844 transactions
    BlobVersionedHashes,
    // Only for EIP-4844 transactions
    MaxFeePerBlobGas,
    Input,

    // ===== TransactionReceipt fields =====
    Bloom,
    Success,
    CumulativeGasUsed,
    Logs,
}

// Note: This index is use to parse the transaction datalake field from the datalake's sampled property.
// It is not used to index the transaction datalake field itself.
impl TransactionDatalakeField {
    pub fn from_index(index: u8) -> Result<Self> {
        match index {
            0 => Ok(TransactionDatalakeField::ChainId),
            1 => Ok(TransactionDatalakeField::Nonce),
            2 => Ok(TransactionDatalakeField::GasPrice),
            3 => Ok(TransactionDatalakeField::GasLimit),
            4 => Ok(TransactionDatalakeField::To),
            5 => Ok(TransactionDatalakeField::Value),
            6 => Ok(TransactionDatalakeField::Input),
            7 => Ok(TransactionDatalakeField::MaxFeePerGas),
            8 => Ok(TransactionDatalakeField::MaxPriorityFeePerGas),
            9 => Ok(TransactionDatalakeField::AccessList),
            10 => Ok(TransactionDatalakeField::BlobVersionedHashes),
            11 => Ok(TransactionDatalakeField::MaxFeePerBlobGas),
            12 => Ok(TransactionDatalakeField::Bloom),
            13 => Ok(TransactionDatalakeField::Success),
            14 => Ok(TransactionDatalakeField::CumulativeGasUsed),
            15 => Ok(TransactionDatalakeField::Logs),
            _ => bail!("Invalid transaction datalake field index"),
        }
    }

    pub fn to_index(&self) -> u8 {
        match self {
            TransactionDatalakeField::ChainId => 0,
            TransactionDatalakeField::Nonce => 1,
            TransactionDatalakeField::GasPrice => 2,
            TransactionDatalakeField::GasLimit => 3,
            TransactionDatalakeField::To => 4,
            TransactionDatalakeField::Value => 5,
            TransactionDatalakeField::Input => 6,
            TransactionDatalakeField::MaxFeePerGas => 7,
            TransactionDatalakeField::MaxPriorityFeePerGas => 8,
            TransactionDatalakeField::AccessList => 9,
            TransactionDatalakeField::BlobVersionedHashes => 10,
            TransactionDatalakeField::MaxFeePerBlobGas => 11,
            TransactionDatalakeField::Bloom => 12,
            TransactionDatalakeField::Success => 13,
            TransactionDatalakeField::CumulativeGasUsed => 14,
            TransactionDatalakeField::Logs => 15,
        }
    }

    pub fn parse_collection(&self) -> TransactionsCollection {
        match self {
            TransactionDatalakeField::ChainId
            | TransactionDatalakeField::Nonce
            | TransactionDatalakeField::GasPrice
            | TransactionDatalakeField::GasLimit
            | TransactionDatalakeField::To
            | TransactionDatalakeField::Value
            | TransactionDatalakeField::Input
            | TransactionDatalakeField::MaxFeePerGas
            | TransactionDatalakeField::MaxPriorityFeePerGas
            | TransactionDatalakeField::AccessList
            | TransactionDatalakeField::BlobVersionedHashes
            | TransactionDatalakeField::MaxFeePerBlobGas => {
                TransactionsCollection::TransactionsBySender
            }
            TransactionDatalakeField::Bloom
            | TransactionDatalakeField::Success
            | TransactionDatalakeField::CumulativeGasUsed
            | TransactionDatalakeField::Logs => {
                TransactionsCollection::TranasactionReceiptsBySender
            }
        }
    }
}

impl FromStr for TransactionDatalakeField {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "CHAIN_ID" => Ok(TransactionDatalakeField::ChainId),
            "NONCE" => Ok(TransactionDatalakeField::Nonce),
            "GAS_PRICE" => Ok(TransactionDatalakeField::GasPrice),
            "GAS_LIMIT" => Ok(TransactionDatalakeField::GasLimit),
            "TO" => Ok(TransactionDatalakeField::To),
            "VALUE" => Ok(TransactionDatalakeField::Value),
            "INPUT" => Ok(TransactionDatalakeField::Input),
            "MAX_FEE_PER_GAS" => Ok(TransactionDatalakeField::MaxFeePerGas),
            "MAX_PRIORITY_FEE_PER_GAS" => Ok(TransactionDatalakeField::MaxPriorityFeePerGas),
            "ACCESS_LIST" => Ok(TransactionDatalakeField::AccessList),
            "BLOB_VERSIONED_HASHES" => Ok(TransactionDatalakeField::BlobVersionedHashes),
            "MAX_FEE_PER_BLOB_GAS" => Ok(TransactionDatalakeField::MaxFeePerBlobGas),
            "BLOOM" => Ok(TransactionDatalakeField::Bloom),
            "SUCCESS" => Ok(TransactionDatalakeField::Success),
            "CUMULATIVE_GAS_USED" => Ok(TransactionDatalakeField::CumulativeGasUsed),
            "LOGS" => Ok(TransactionDatalakeField::Logs),
            _ => bail!("Unknown transaction datalake field"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LegacyTransactionField {
    ChainId = 0,
    Nonce = 1,
    GasPrice = 2,
    GasLimit = 3,
    To = 4,
    Value = 5,
    Input = 6,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Eip2930TransactionField {
    ChainId = 0,
    Nonce = 1,
    GasPrice = 2,
    GasLimit = 3,
    To,
    Value,
    AccessList,
    Input,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Eip1559TransactionField {
    ChainId,
    Nonce,
    GasLimit,
    MaxFeePerGas,
    MaxPriorityFeePerGas,
    To,
    Value,
    AccessList,
    Input,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Eip4844TransactionField {
    ChainId,
    Nonce,
    GasLimit,
    MaxFeePerGas,
    MaxPriorityFeePerGas,
    To,
    Value,
    AccessList,
    BlobVersionedHashes,
    MaxFeePerBlobGas,
    Input,
}
