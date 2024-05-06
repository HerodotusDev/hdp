use std::str::FromStr;

use alloy_primitives::hex;
use anyhow::{bail, Result};
use eth_trie_proofs::{tx::ConsensusTx, tx_receipt::ConsensusTxReceipt};

use crate::{datalake::DatalakeField, utils::bytes_to_hex_string};

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

impl TransactionField {
    pub fn variants() -> Vec<String> {
        vec![
            "NONCE".to_string(),
            "GAS_PRICE".to_string(),
            "GAS_LIMIT".to_string(),
            "TO".to_string(),
            "VALUE".to_string(),
            "INPUT".to_string(),
            "V".to_string(),
            "R".to_string(),
            "S".to_string(),
            "CHAIN_ID".to_string(),
            "ACCESS_LIST".to_string(),
            "MAX_FEE_PER_GAS".to_string(),
            "MAX_PRIORITY_FEE_PER_GAS".to_string(),
            "BLOB_VERSIONED_HASHES".to_string(),
            "MAX_FEE_PER_BLOB_GAS".to_string(),
        ]
    }
}

// Note: This index is use to parse the transaction datalake field from the datalake's sampled property.
// It is not used to index the transaction datalake field itself.
impl DatalakeField for TransactionField {
    fn from_index(index: u8) -> Result<Self> {
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

    fn to_index(&self) -> u8 {
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

    fn decode_field_from_rlp(&self, rlp: &str) -> String {
        let raw_tx = ConsensusTx::rlp_decode(hex::decode(rlp).unwrap().as_slice()).unwrap();
        match self {
            TransactionField::Nonce => raw_tx.nonce().to_string(),
            TransactionField::GasPrice => raw_tx.gas_price().map(|x| x.to_string()).unwrap(),
            TransactionField::GasLimit => raw_tx.gas_limit().to_string(),
            TransactionField::To => raw_tx.to().to().map(|x| x.to_string()).unwrap(),
            TransactionField::Value => raw_tx.value().to_string(),
            TransactionField::Input => bytes_to_hex_string(raw_tx.input()),
            TransactionField::V => raw_tx.v().to_string(),
            TransactionField::R => raw_tx.r().to_string(),
            TransactionField::S => raw_tx.s().to_string(),
            TransactionField::ChainId => raw_tx.chain_id().map(|x| x.to_string()).unwrap(),
            // TODO:  string should be properly rlp encoded
            TransactionField::AccessList => raw_tx
                .access_list()
                .map(|_| "access_list".to_string())
                .unwrap(),
            TransactionField::MaxFeePerGas => {
                raw_tx.max_fee_per_gas().map(|x| x.to_string()).unwrap()
            }
            TransactionField::MaxPriorityFeePerGas => raw_tx
                .max_priority_fee_per_gas()
                .map(|x| x.to_string())
                .unwrap(),
            // TODO:  string should be properly rlp encoded
            TransactionField::BlobVersionedHashes => raw_tx
                .blob_versioned_hashes()
                .map(|x| x[0].to_string())
                .unwrap(),
            TransactionField::MaxFeePerBlobGas => raw_tx
                .max_fee_per_blob_gas()
                .map(|x| x.to_string())
                .unwrap(),
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

impl ToString for TransactionField {
    fn to_string(&self) -> String {
        match self {
            TransactionField::Nonce => "NONCE".to_string(),
            TransactionField::GasPrice => "GAS_PRICE".to_string(),
            TransactionField::GasLimit => "GAS_LIMIT".to_string(),
            TransactionField::To => "TO".to_string(),
            TransactionField::Value => "VALUE".to_string(),
            TransactionField::Input => "INPUT".to_string(),
            TransactionField::V => "V".to_string(),
            TransactionField::R => "R".to_string(),
            TransactionField::S => "S".to_string(),
            TransactionField::ChainId => "CHAIN_ID".to_string(),
            TransactionField::AccessList => "ACCESS_LIST".to_string(),
            TransactionField::MaxFeePerGas => "MAX_FEE_PER_GAS".to_string(),
            TransactionField::MaxPriorityFeePerGas => "MAX_PRIORITY_FEE_PER_GAS".to_string(),
            TransactionField::BlobVersionedHashes => "BLOB_VERSIONED_HASHES".to_string(),
            TransactionField::MaxFeePerBlobGas => "MAX_FEE_PER_BLOB_GAS".to_string(),
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

impl TransactionReceiptField {
    pub fn variants() -> Vec<String> {
        vec![
            "SUCCESS".to_string(),
            "CUMULATIVE_GAS_USED".to_string(),
            "LOGS".to_string(),
            "BLOOM".to_string(),
        ]
    }
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

impl ToString for TransactionReceiptField {
    fn to_string(&self) -> String {
        match self {
            TransactionReceiptField::Success => "SUCCESS".to_string(),
            TransactionReceiptField::CumulativeGasUsed => "CUMULATIVE_GAS_USED".to_string(),
            TransactionReceiptField::Logs => "LOGS".to_string(),
            TransactionReceiptField::Bloom => "BLOOM".to_string(),
        }
    }
}

impl DatalakeField for TransactionReceiptField {
    fn to_index(&self) -> u8 {
        match self {
            TransactionReceiptField::Success => 0,
            TransactionReceiptField::CumulativeGasUsed => 1,
            TransactionReceiptField::Logs => 2,
            TransactionReceiptField::Bloom => 3,
        }
    }

    fn from_index(index: u8) -> Result<Self> {
        match index {
            0 => Ok(TransactionReceiptField::Success),
            1 => Ok(TransactionReceiptField::CumulativeGasUsed),
            2 => Ok(TransactionReceiptField::Logs),
            3 => Ok(TransactionReceiptField::Bloom),
            _ => bail!("Invalid transaction receipt field index"),
        }
    }

    fn decode_field_from_rlp(&self, rlp: &str) -> String {
        let raw_tx_receipt =
            ConsensusTxReceipt::rlp_decode(hex::decode(rlp).unwrap().as_slice()).unwrap();

        match self {
            TransactionReceiptField::Success => match raw_tx_receipt.success() {
                true => "1".to_string(),
                false => "0".to_string(),
            },
            TransactionReceiptField::CumulativeGasUsed => {
                raw_tx_receipt.cumulative_gas_used().to_string()
            }
            // TODO: string should be properly rlp encoded
            TransactionReceiptField::Logs => "logs".to_string(),
            TransactionReceiptField::Bloom => "bloom".to_string(),
        }
    }
}
