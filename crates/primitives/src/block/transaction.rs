use std::str::FromStr;

use anyhow::{bail, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum TransactionField {
    Hash,
    Nonce,
    BlockHash,
    BlockNumber,
    TransactionIndex,
    From,
    To,
    Value,
    GasPrice,
    Gas,
    Input,
    Signature,
    ChainId,
    BlobVersionedHashes,
    AccessList,
    TransactionType,
    MaxFeePerGas,
    MaxPriorityFeePerGas,
    MaxFeePerBlobGas,
    Other,
}

impl FromStr for TransactionField {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "hash" => Ok(TransactionField::Hash),
            "nonce" => Ok(TransactionField::Nonce),
            "block_hash" => Ok(TransactionField::BlockHash),
            "block_number" => Ok(TransactionField::BlockNumber),
            "transaction_index" => Ok(TransactionField::TransactionIndex),
            "from" => Ok(TransactionField::From),
            "to" => Ok(TransactionField::To),
            "value" => Ok(TransactionField::Value),
            "gas_price" => Ok(TransactionField::GasPrice),
            "gas" => Ok(TransactionField::Gas),
            "input" => Ok(TransactionField::Input),
            "signature" => Ok(TransactionField::Signature),
            "chain_id" => Ok(TransactionField::ChainId),
            "blob_versioned_hashes" => Ok(TransactionField::BlobVersionedHashes),
            "access_list" => Ok(TransactionField::AccessList),
            "transaction_type" => Ok(TransactionField::TransactionType),
            "max_fee_per_gas" => Ok(TransactionField::MaxFeePerGas),
            "max_priority_fee_per_gas" => Ok(TransactionField::MaxPriorityFeePerGas),
            "max_fee_per_blob_gas" => Ok(TransactionField::MaxFeePerBlobGas),
            "other" => Ok(TransactionField::Other),
            _ => bail!("Unknown account field"),
        }
    }
}

impl ToString for TransactionField {
    fn to_string(&self) -> String {
        match self {
            TransactionField::Hash => "hash".to_string(),
            TransactionField::Nonce => "nonce".to_string(),
            TransactionField::BlockHash => "block_hash".to_string(),
            TransactionField::BlockNumber => "block_number".to_string(),
            TransactionField::TransactionIndex => "transaction_index".to_string(),
            TransactionField::From => "from".to_string(),
            TransactionField::To => "to".to_string(),
            TransactionField::Value => "value".to_string(),
            TransactionField::GasPrice => "gas_price".to_string(),
            TransactionField::Gas => "gas".to_string(),
            TransactionField::Input => "input".to_string(),
            TransactionField::Signature => "signature".to_string(),
            TransactionField::ChainId => "chain_id".to_string(),
            TransactionField::BlobVersionedHashes => "blob_versioned_hashes".to_string(),
            TransactionField::AccessList => "access_list".to_string(),
            TransactionField::TransactionType => "transaction_type".to_string(),
            TransactionField::MaxFeePerGas => "max_fee_per_gas".to_string(),
            TransactionField::MaxPriorityFeePerGas => "max_priority_fee_per_gas".to_string(),
            TransactionField::MaxFeePerBlobGas => "max_fee_per_blob_gas".to_string(),
            TransactionField::Other => "other".to_string(),
        }
    }
}

impl TransactionField {
    pub fn from_index(index: u8) -> Result<Self> {
        match index {
            0 => Ok(TransactionField::Hash),
            1 => Ok(TransactionField::Nonce),
            2 => Ok(TransactionField::BlockHash),
            3 => Ok(TransactionField::BlockNumber),
            4 => Ok(TransactionField::TransactionIndex),
            5 => Ok(TransactionField::From),
            6 => Ok(TransactionField::To),
            7 => Ok(TransactionField::Value),
            8 => Ok(TransactionField::GasPrice),
            9 => Ok(TransactionField::Gas),
            10 => Ok(TransactionField::Input),
            11 => Ok(TransactionField::Signature),
            12 => Ok(TransactionField::ChainId),
            13 => Ok(TransactionField::BlobVersionedHashes),
            14 => Ok(TransactionField::AccessList),
            15 => Ok(TransactionField::TransactionType),
            16 => Ok(TransactionField::MaxFeePerGas),
            17 => Ok(TransactionField::MaxPriorityFeePerGas),
            18 => Ok(TransactionField::MaxFeePerBlobGas),
            19 => Ok(TransactionField::Other),
            _ => bail!("Invalid transaction field"),
        }
    }

    pub fn to_index(&self) -> u8 {
        match self {
            TransactionField::Hash => 0,
            TransactionField::Nonce => 1,
            TransactionField::BlockHash => 2,
            TransactionField::BlockNumber => 3,
            TransactionField::TransactionIndex => 4,
            TransactionField::From => 5,
            TransactionField::To => 6,
            TransactionField::Value => 7,
            TransactionField::GasPrice => 8,
            TransactionField::Gas => 9,
            TransactionField::Input => 10,
            TransactionField::Signature => 11,
            TransactionField::ChainId => 12,
            TransactionField::BlobVersionedHashes => 13,
            TransactionField::AccessList => 14,
            TransactionField::TransactionType => 15,
            TransactionField::MaxFeePerGas => 16,
            TransactionField::MaxPriorityFeePerGas => 17,
            TransactionField::MaxFeePerBlobGas => 18,
            TransactionField::Other => 19,
        }
    }
}
