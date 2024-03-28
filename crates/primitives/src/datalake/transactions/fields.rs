use std::str::FromStr;

use anyhow::{bail, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionField {
    // ===== Transaction fields =====
    Nonce,
    GasPrice,
    GasLimit,
    To,
    Value,
    Input,
    V,
    R,
    S,
    ChainId,
    // Not for legacy transactions
    AccessList,

    // EIP-1559 transactions and EIP-4844 transactions
    MaxFeePerGas,
    // EIP-1559 transactions and EIP-4844 transactions
    MaxPriorityFeePerGas,

    // Only for EIP-4844 transactions
    BlobVersionedHashes,
    // Only for EIP-4844 transactions
    MaxFeePerBlobGas,
}

// Note: This index is use to parse the transaction datalake field from the datalake's sampled property.
// It is not used to index the transaction datalake field itself.
impl TransactionField {
    pub fn from_index(index: u8) -> Result<Self> {
        match index {
            0 => Ok(TransactionField::Nonce),
            1 => Ok(TransactionField::GasPrice),
            2 => Ok(TransactionField::GasLimit),
            3 => Ok(TransactionField::To),
            4 => Ok(TransactionField::Value),
            5 => Ok(TransactionField::Input),
            6 => Ok(TransactionField::V),
            7 => Ok(TransactionField::R),
            8 => Ok(TransactionField::S),
            9 => Ok(TransactionField::ChainId),
            10 => Ok(TransactionField::AccessList),
            11 => Ok(TransactionField::MaxFeePerGas),
            12 => Ok(TransactionField::MaxPriorityFeePerGas),
            13 => Ok(TransactionField::BlobVersionedHashes),
            14 => Ok(TransactionField::MaxFeePerBlobGas),
            _ => bail!("Invalid transaction field index"),
        }
    }

    pub fn to_index(&self) -> u8 {
        match self {
            TransactionField::Nonce => 0,
            TransactionField::GasPrice => 1,
            TransactionField::GasLimit => 2,
            TransactionField::To => 3,
            TransactionField::Value => 4,
            TransactionField::Input => 5,
            TransactionField::V => 6,
            TransactionField::R => 7,
            TransactionField::S => 8,
            TransactionField::ChainId => 9,
            TransactionField::AccessList => 10,
            TransactionField::MaxFeePerGas => 11,
            TransactionField::MaxPriorityFeePerGas => 12,
            TransactionField::BlobVersionedHashes => 13,
            TransactionField::MaxFeePerBlobGas => 14,
        }
    }
}

impl FromStr for TransactionField {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "NONCE" => Ok(TransactionField::Nonce),
            "GAS_PRICE" => Ok(TransactionField::GasPrice),
            "GAS_LIMIT" => Ok(TransactionField::GasLimit),
            "TO" => Ok(TransactionField::To),
            "VALUE" => Ok(TransactionField::Value),
            "INPUT" => Ok(TransactionField::Input),
            "V" => Ok(TransactionField::V),
            "R" => Ok(TransactionField::R),
            "S" => Ok(TransactionField::S),
            "CHAIN_ID" => Ok(TransactionField::ChainId),
            "ACCESS_LIST" => Ok(TransactionField::AccessList),
            "MAX_FEE_PER_GAS" => Ok(TransactionField::MaxFeePerGas),
            "MAX_PRIORITY_FEE_PER_GAS" => Ok(TransactionField::MaxPriorityFeePerGas),
            "BLOB_VERSIONED_HASHES" => Ok(TransactionField::BlobVersionedHashes),
            "MAX_FEE_PER_BLOB_GAS" => Ok(TransactionField::MaxFeePerBlobGas),
            _ => bail!("Unknown transaction datalake field"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionReceiptField {
    Success,
    CumulativeGasUsed,
    Logs,
    Bloom,
}

impl FromStr for TransactionReceiptField {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "SUCCESS" => Ok(TransactionReceiptField::Success),
            "CUMULATIVE_GAS_USED" => Ok(TransactionReceiptField::CumulativeGasUsed),
            "LOGS" => Ok(TransactionReceiptField::Logs),
            "BLOOM" => Ok(TransactionReceiptField::Bloom),
            _ => bail!("Unknown transaction receipt field"),
        }
    }
}

impl TransactionReceiptField {
    pub fn to_index(&self) -> u8 {
        match self {
            TransactionReceiptField::Success => 0,
            TransactionReceiptField::CumulativeGasUsed => 1,
            TransactionReceiptField::Logs => 2,
            TransactionReceiptField::Bloom => 3,
        }
    }

    pub fn from_index(index: u8) -> Result<Self> {
        match index {
            0 => Ok(TransactionReceiptField::Success),
            1 => Ok(TransactionReceiptField::CumulativeGasUsed),
            2 => Ok(TransactionReceiptField::Logs),
            3 => Ok(TransactionReceiptField::Bloom),
            _ => bail!("Invalid transaction receipt field index"),
        }
    }
}
