use std::{fmt::Display, str::FromStr};

use alloy::{consensus::Eip658Value, primitives::U256};
use anyhow::{bail, Result};
use eth_trie_proofs::{tx::ConsensusTx, tx_receipt::ConsensusTxReceipt};

use crate::task::datalake::DatalakeField;

#[derive(Debug, Clone, PartialEq, Eq)]
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

    /// This function is for generating random TransactionField for testing purposes.
    pub fn integer_variants_index(index: u8) -> Self {
        match index {
            0 => TransactionField::Nonce,
            1 => TransactionField::GasPrice,
            2 => TransactionField::GasLimit,
            3 => TransactionField::ChainId,
            4 => TransactionField::MaxFeePerGas,
            5 => TransactionField::MaxPriorityFeePerGas,
            6 => TransactionField::MaxFeePerBlobGas,
            _ => unreachable!(),
        }
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

    fn decode_field_from_rlp(&self, rlp: &[u8]) -> U256 {
        let raw_tx = ConsensusTx::rlp_decode(rlp).unwrap();
        match self {
            TransactionField::Nonce => U256::from(raw_tx.nonce()),
            TransactionField::GasPrice => {
                U256::from(raw_tx.gas_price().expect("gas price does not exist"))
            }
            TransactionField::GasLimit => U256::from(raw_tx.gas_limit()),
            TransactionField::To => U256::from_str_radix(
                &raw_tx.to().to().expect("to does not exist").to_string(),
                16,
            )
            .unwrap(),
            TransactionField::Value => U256::from(raw_tx.value()),
            TransactionField::Input => U256::from_be_slice(raw_tx.input()),
            TransactionField::V => U256::from(raw_tx.v()),
            TransactionField::R => U256::from(raw_tx.r()),
            TransactionField::S => U256::from(raw_tx.s()),
            TransactionField::ChainId => {
                U256::from(raw_tx.chain_id().expect("chain id does not exist"))
            }
            // TODO:  string should be properly rlp encoded
            TransactionField::AccessList => todo!("access list cannot parse into u256"),
            TransactionField::MaxFeePerGas => U256::from(
                raw_tx
                    .max_fee_per_gas()
                    .expect("max fee per gas does not exist"),
            ),
            TransactionField::MaxPriorityFeePerGas => U256::from(
                raw_tx
                    .max_priority_fee_per_gas()
                    .expect("max priority fee per gas does not exist"),
            ),
            TransactionField::BlobVersionedHashes => raw_tx
                .blob_versioned_hashes()
                .expect("blob versioned hashes does not exist")[0]
                .into(),
            TransactionField::MaxFeePerBlobGas => U256::from(
                raw_tx
                    .max_fee_per_blob_gas()
                    .expect("max fee per blob gas does not exist"),
            ),
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

impl Display for TransactionField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionField::Nonce => write!(f, "NONCE"),
            TransactionField::GasPrice => write!(f, "GAS_PRICE"),
            TransactionField::GasLimit => write!(f, "GAS_LIMIT"),
            TransactionField::To => write!(f, "TO"),
            TransactionField::Value => write!(f, "VALUE"),
            TransactionField::Input => write!(f, "INPUT"),
            TransactionField::V => write!(f, "V"),
            TransactionField::R => write!(f, "R"),
            TransactionField::S => write!(f, "S"),
            TransactionField::ChainId => write!(f, "CHAIN_ID"),
            TransactionField::AccessList => write!(f, "ACCESS_LIST"),
            TransactionField::MaxFeePerGas => write!(f, "MAX_FEE_PER_GAS"),
            TransactionField::MaxPriorityFeePerGas => write!(f, "MAX_PRIORITY_FEE_PER_GAS"),
            TransactionField::BlobVersionedHashes => write!(f, "BLOB_VERSIONED_HASHES"),
            TransactionField::MaxFeePerBlobGas => write!(f, "MAX_FEE_PER_BLOB_GAS"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl Display for TransactionReceiptField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionReceiptField::Success => write!(f, "SUCCESS"),
            TransactionReceiptField::CumulativeGasUsed => write!(f, "CUMULATIVE_GAS_USED"),
            TransactionReceiptField::Logs => write!(f, "LOGS"),
            TransactionReceiptField::Bloom => write!(f, "BLOOM"),
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

    fn decode_field_from_rlp(&self, rlp: &[u8]) -> U256 {
        let raw_tx_receipt = ConsensusTxReceipt::rlp_decode(rlp).unwrap();

        match self {
            TransactionReceiptField::Success => match raw_tx_receipt.status() {
                Eip658Value::Eip658(bool) => U256::from(*bool as u8),
                Eip658Value::PostState(state) => (*state).into(),
            },
            TransactionReceiptField::CumulativeGasUsed => {
                U256::from(raw_tx_receipt.cumulative_gas_used())
            }
            // TODO: string should be properly rlp encoded
            TransactionReceiptField::Logs => U256::from(raw_tx_receipt.logs().len()),
            TransactionReceiptField::Bloom => U256::from(raw_tx_receipt.bloom().len()),
        }
    }
}
