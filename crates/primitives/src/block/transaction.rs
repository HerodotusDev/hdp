use std::str::FromStr;

use anyhow::bail;

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
    pub fn from_index(index: u8) -> Option<TransactionField> {
        match index {
            0 => Some(TransactionField::Hash),
            1 => Some(TransactionField::Nonce),
            2 => Some(TransactionField::BlockHash),
            3 => Some(TransactionField::BlockNumber),
            4 => Some(TransactionField::TransactionIndex),
            5 => Some(TransactionField::From),
            6 => Some(TransactionField::To),
            7 => Some(TransactionField::Value),
            8 => Some(TransactionField::GasPrice),
            9 => Some(TransactionField::Gas),
            10 => Some(TransactionField::Input),
            11 => Some(TransactionField::Signature),
            12 => Some(TransactionField::ChainId),
            13 => Some(TransactionField::BlobVersionedHashes),
            14 => Some(TransactionField::AccessList),
            15 => Some(TransactionField::TransactionType),
            16 => Some(TransactionField::MaxFeePerGas),
            17 => Some(TransactionField::MaxPriorityFeePerGas),
            18 => Some(TransactionField::MaxFeePerBlobGas),
            _ => Some(TransactionField::Other),
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
