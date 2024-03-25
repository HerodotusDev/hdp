use std::str::FromStr;

use anyhow::{bail, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionReceiptField {
    TransactionHash,
    TransactionIndex,
    BlockHash,
    BlockNumber,
    CumulativeGasUsed,
    GasUsed,
    EffectiveGasPrice,
    BlobGasUsed,
    BlobGasPrice,
    From,
    To,
    ContractAddress,
    Logs,
    LogsBloom,
    StateRoot,
    StatusCode,
    TransactionType,
    Other,
}

impl FromStr for TransactionReceiptField {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "transaction_hash" => Ok(TransactionReceiptField::TransactionHash),
            "transaction_index" => Ok(TransactionReceiptField::TransactionIndex),
            "block_hash" => Ok(TransactionReceiptField::BlockHash),
            "block_number" => Ok(TransactionReceiptField::BlockNumber),
            "cumulative_gas_used" => Ok(TransactionReceiptField::CumulativeGasUsed),
            "gas_used" => Ok(TransactionReceiptField::GasUsed),
            "effective_gas_price" => Ok(TransactionReceiptField::EffectiveGasPrice),
            "blob_gas_used" => Ok(TransactionReceiptField::BlobGasUsed),
            "blob_gas_price" => Ok(TransactionReceiptField::BlobGasPrice),
            "from" => Ok(TransactionReceiptField::From),
            "to" => Ok(TransactionReceiptField::To),
            "contract_address" => Ok(TransactionReceiptField::ContractAddress),
            "logs" => Ok(TransactionReceiptField::Logs),
            "logs_bloom" => Ok(TransactionReceiptField::LogsBloom),
            "state_root" => Ok(TransactionReceiptField::StateRoot),
            "status_code" => Ok(TransactionReceiptField::StatusCode),
            "transaction_type" => Ok(TransactionReceiptField::TransactionType),
            "other" => Ok(TransactionReceiptField::Other),
            _ => bail!("Unknown transaction receipt field"),
        }
    }
}

impl ToString for TransactionReceiptField {
    fn to_string(&self) -> String {
        match self {
            TransactionReceiptField::TransactionHash => "transaction_hash".to_string(),
            TransactionReceiptField::TransactionIndex => "transaction_index".to_string(),
            TransactionReceiptField::BlockHash => "block_hash".to_string(),
            TransactionReceiptField::BlockNumber => "block_number".to_string(),
            TransactionReceiptField::CumulativeGasUsed => "cumulative_gas_used".to_string(),
            TransactionReceiptField::GasUsed => "gas_used".to_string(),
            TransactionReceiptField::EffectiveGasPrice => "effective_gas_price".to_string(),
            TransactionReceiptField::BlobGasUsed => "blob_gas_used".to_string(),
            TransactionReceiptField::BlobGasPrice => "blob_gas_price".to_string(),
            TransactionReceiptField::From => "from".to_string(),
            TransactionReceiptField::To => "to".to_string(),
            TransactionReceiptField::ContractAddress => "contract_address".to_string(),
            TransactionReceiptField::Logs => "logs".to_string(),
            TransactionReceiptField::LogsBloom => "logs_bloom".to_string(),
            TransactionReceiptField::StateRoot => "state_root".to_string(),
            TransactionReceiptField::StatusCode => "status_code".to_string(),
            TransactionReceiptField::TransactionType => "transaction_type".to_string(),
            TransactionReceiptField::Other => "other".to_string(),
        }
    }
}

impl TransactionReceiptField {
    pub fn from_index(index: u8) -> Result<Self> {
        match index {
            0 => Ok(TransactionReceiptField::TransactionHash),
            1 => Ok(TransactionReceiptField::TransactionIndex),
            2 => Ok(TransactionReceiptField::BlockHash),
            3 => Ok(TransactionReceiptField::BlockNumber),
            4 => Ok(TransactionReceiptField::CumulativeGasUsed),
            5 => Ok(TransactionReceiptField::GasUsed),
            6 => Ok(TransactionReceiptField::EffectiveGasPrice),
            7 => Ok(TransactionReceiptField::BlobGasUsed),
            8 => Ok(TransactionReceiptField::BlobGasPrice),
            9 => Ok(TransactionReceiptField::From),
            10 => Ok(TransactionReceiptField::To),
            11 => Ok(TransactionReceiptField::ContractAddress),
            12 => Ok(TransactionReceiptField::Logs),
            13 => Ok(TransactionReceiptField::LogsBloom),
            14 => Ok(TransactionReceiptField::StateRoot),
            15 => Ok(TransactionReceiptField::StatusCode),
            16 => Ok(TransactionReceiptField::TransactionType),
            17 => Ok(TransactionReceiptField::Other),
            _ => bail!("Invalid transaction receipt field"),
        }
    }

    pub fn to_index(&self) -> u8 {
        match self {
            TransactionReceiptField::TransactionHash => 0,
            TransactionReceiptField::TransactionIndex => 1,
            TransactionReceiptField::BlockHash => 2,
            TransactionReceiptField::BlockNumber => 3,
            TransactionReceiptField::CumulativeGasUsed => 4,
            TransactionReceiptField::GasUsed => 5,
            TransactionReceiptField::EffectiveGasPrice => 6,
            TransactionReceiptField::BlobGasUsed => 7,
            TransactionReceiptField::BlobGasPrice => 8,
            TransactionReceiptField::From => 9,
            TransactionReceiptField::To => 10,
            TransactionReceiptField::ContractAddress => 11,
            TransactionReceiptField::Logs => 12,
            TransactionReceiptField::LogsBloom => 13,
            TransactionReceiptField::StateRoot => 14,
            TransactionReceiptField::StatusCode => 15,
            TransactionReceiptField::TransactionType => 16,
            TransactionReceiptField::Other => 17,
        }
    }
}
