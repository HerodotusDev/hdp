//! This module defines the fields that can be queried from a block or account.
//! The fields are defined as enums, and can be converted to and from their string representation.
//! It is meant to be used in the `BlockSampled` struct, which is used to query fields from a block or account.

use std::{fmt::Display, str::FromStr};

use alloy::primitives::U256;
use anyhow::{bail, Result};

use crate::{
    block::{account::Account, header::Header},
    datalake::DatalakeField,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeaderField {
    ParentHash,
    OmmerHash,
    Beneficiary,
    StateRoot,
    TransactionsRoot,
    ReceiptsRoot,
    LogsBloom,
    Difficulty,
    Number,
    GasLimit,
    GasUsed,
    Timestamp,
    ExtraData,
    MixHash,
    Nonce,
    BaseFeePerGas,
    WithdrawalsRoot,
    BlobGasUsed,
    ExcessBlobGas,
    ParentBeaconBlockRoot,
}

impl HeaderField {
    pub fn variants() -> Vec<String> {
        vec![
            "PARENT_HASH".to_string(),
            "OMMERS_HASH".to_string(),
            "BENEFICIARY".to_string(),
            "STATE_ROOT".to_string(),
            "TRANSACTIONS_ROOT".to_string(),
            "RECEIPTS_ROOT".to_string(),
            "LOGS_BLOOM".to_string(),
            "DIFFICULTY".to_string(),
            "NUMBER".to_string(),
            "GAS_LIMIT".to_string(),
            "GAS_USED".to_string(),
            "TIMESTAMP".to_string(),
            "EXTRA_DATA".to_string(),
            "MIX_HASH".to_string(),
            "NONCE".to_string(),
            "BASE_FEE_PER_GAS".to_string(),
            "WITHDRAWALS_ROOT".to_string(),
            "BLOB_GAS_USED".to_string(),
            "EXCESS_BLOB_GAS".to_string(),
            "PARENT_BEACON_BLOCK_ROOT".to_string(),
        ]
    }

    pub fn integer_variants_index(index: u8) -> Self {
        match index {
            0 => HeaderField::Difficulty,
            1 => HeaderField::Number,
            2 => HeaderField::GasLimit,
            3 => HeaderField::GasUsed,
            4 => HeaderField::Timestamp,
            5 => HeaderField::Nonce,
            6 => HeaderField::BaseFeePerGas,
            7 => HeaderField::BlobGasUsed,
            8 => HeaderField::ExcessBlobGas,
            _ => unreachable!(),
        }
    }
}

impl DatalakeField for HeaderField {
    fn from_index(index: u8) -> Result<Self> {
        match index {
            0 => Ok(HeaderField::ParentHash),
            1 => Ok(HeaderField::OmmerHash),
            2 => Ok(HeaderField::Beneficiary),
            3 => Ok(HeaderField::StateRoot),
            4 => Ok(HeaderField::TransactionsRoot),
            5 => Ok(HeaderField::ReceiptsRoot),
            6 => Ok(HeaderField::LogsBloom),
            7 => Ok(HeaderField::Difficulty),
            8 => Ok(HeaderField::Number),
            9 => Ok(HeaderField::GasLimit),
            10 => Ok(HeaderField::GasUsed),
            11 => Ok(HeaderField::Timestamp),
            12 => Ok(HeaderField::ExtraData),
            13 => Ok(HeaderField::MixHash),
            14 => Ok(HeaderField::Nonce),
            15 => Ok(HeaderField::BaseFeePerGas),
            16 => Ok(HeaderField::WithdrawalsRoot),
            17 => Ok(HeaderField::BlobGasUsed),
            18 => Ok(HeaderField::ExcessBlobGas),
            19 => Ok(HeaderField::ParentBeaconBlockRoot),
            _ => bail!("Unknown header field"),
        }
    }

    fn to_index(&self) -> u8 {
        match self {
            HeaderField::ParentHash => 0,
            HeaderField::OmmerHash => 1,
            HeaderField::Beneficiary => 2,
            HeaderField::StateRoot => 3,
            HeaderField::TransactionsRoot => 4,
            HeaderField::ReceiptsRoot => 5,
            HeaderField::LogsBloom => 6,
            HeaderField::Difficulty => 7,
            HeaderField::Number => 8,
            HeaderField::GasLimit => 9,
            HeaderField::GasUsed => 10,
            HeaderField::Timestamp => 11,
            HeaderField::ExtraData => 12,
            HeaderField::MixHash => 13,
            HeaderField::Nonce => 14,
            HeaderField::BaseFeePerGas => 15,
            HeaderField::WithdrawalsRoot => 16,
            HeaderField::BlobGasUsed => 17,
            HeaderField::ExcessBlobGas => 18,
            HeaderField::ParentBeaconBlockRoot => 19,
        }
    }

    fn decode_field_from_rlp(&self, header_rlp: &[u8]) -> U256 {
        let decoded = <Header>::rlp_decode(header_rlp);

        match self {
            HeaderField::ParentHash => decoded.parent_hash.into(),
            HeaderField::OmmerHash => decoded.ommers_hash.into(),
            HeaderField::Beneficiary => {
                U256::from_str_radix(&decoded.beneficiary.to_string(), 16).unwrap()
            }
            HeaderField::StateRoot => decoded.state_root.into(),
            HeaderField::TransactionsRoot => decoded.transactions_root.into(),
            HeaderField::ReceiptsRoot => decoded.receipts_root.into(),
            HeaderField::LogsBloom => U256::from_str_radix(&decoded.logs_bloom.to_string(), 16)
                .expect("logs bloom does not match U256"),
            HeaderField::Difficulty => U256::from(decoded.difficulty),
            HeaderField::Number => U256::from(decoded.number),
            HeaderField::GasLimit => U256::from(decoded.gas_limit),
            HeaderField::GasUsed => U256::from(decoded.gas_used),
            HeaderField::Timestamp => U256::from(decoded.timestamp),
            HeaderField::ExtraData => todo!("extra data doesn't fit into U256"),
            HeaderField::MixHash => decoded.mix_hash.into(),
            HeaderField::Nonce => U256::from(decoded.nonce),
            HeaderField::BaseFeePerGas => U256::from(
                decoded
                    .base_fee_per_gas
                    .expect("base fee per gas does not exist"),
            ),
            HeaderField::WithdrawalsRoot => decoded
                .withdrawals_root
                .expect("withdrawals root does not exist")
                .into(),
            HeaderField::BlobGasUsed => U256::from(decoded.blob_gas_used.unwrap()),
            HeaderField::ExcessBlobGas => U256::from(decoded.excess_blob_gas.unwrap()),
            HeaderField::ParentBeaconBlockRoot => decoded
                .parent_beacon_block_root
                .expect("parent beacon block root does not exist")
                .into(),
        }
    }
}

impl FromStr for HeaderField {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "PARENT_HASH" => Ok(HeaderField::ParentHash),
            "OMMERS_HASH" => Ok(HeaderField::OmmerHash),
            "BENEFICIARY" => Ok(HeaderField::Beneficiary),
            "STATE_ROOT" => Ok(HeaderField::StateRoot),
            "TRANSACTIONS_ROOT" => Ok(HeaderField::TransactionsRoot),
            "RECEIPTS_ROOT" => Ok(HeaderField::ReceiptsRoot),
            "LOGS_BLOOM" => Ok(HeaderField::LogsBloom),
            "DIFFICULTY" => Ok(HeaderField::Difficulty),
            "NUMBER" => Ok(HeaderField::Number),
            "GAS_LIMIT" => Ok(HeaderField::GasLimit),
            "GAS_USED" => Ok(HeaderField::GasUsed),
            "TIMESTAMP" => Ok(HeaderField::Timestamp),
            "EXTRA_DATA" => Ok(HeaderField::ExtraData),
            "MIX_HASH" => Ok(HeaderField::MixHash),
            "NONCE" => Ok(HeaderField::Nonce),
            "BASE_FEE_PER_GAS" => Ok(HeaderField::BaseFeePerGas),
            "WITHDRAWALS_ROOT" => Ok(HeaderField::WithdrawalsRoot),
            "BLOB_GAS_USED" => Ok(HeaderField::BlobGasUsed),
            "EXCESS_BLOB_GAS" => Ok(HeaderField::ExcessBlobGas),
            "PARENT_BEACON_BLOCK_ROOT" => Ok(HeaderField::ParentBeaconBlockRoot),
            _ => bail!("Unknown header field"),
        }
    }
}

impl Display for HeaderField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HeaderField::ParentHash => write!(f, "PARENT_HASH"),
            HeaderField::OmmerHash => write!(f, "OMMERS_HASH"),
            HeaderField::Beneficiary => write!(f, "BENEFICIARY"),
            HeaderField::StateRoot => write!(f, "STATE_ROOT"),
            HeaderField::TransactionsRoot => write!(f, "TRANSACTIONS_ROOT"),
            HeaderField::ReceiptsRoot => write!(f, "RECEIPTS_ROOT"),
            HeaderField::LogsBloom => write!(f, "LOGS_BLOOM"),
            HeaderField::Difficulty => write!(f, "DIFFICULTY"),
            HeaderField::Number => write!(f, "NUMBER"),
            HeaderField::GasLimit => write!(f, "GAS_LIMIT"),
            HeaderField::GasUsed => write!(f, "GAS_USED"),
            HeaderField::Timestamp => write!(f, "TIMESTAMP"),
            HeaderField::ExtraData => write!(f, "EXTRA_DATA"),
            HeaderField::MixHash => write!(f, "MIX_HASH"),
            HeaderField::Nonce => write!(f, "NONCE"),
            HeaderField::BaseFeePerGas => write!(f, "BASE_FEE_PER_GAS"),
            HeaderField::WithdrawalsRoot => write!(f, "WITHDRAWALS_ROOT"),
            HeaderField::BlobGasUsed => write!(f, "BLOB_GAS_USED"),
            HeaderField::ExcessBlobGas => write!(f, "EXCESS_BLOB_GAS"),
            HeaderField::ParentBeaconBlockRoot => write!(f, "PARENT_BEACON_BLOCK_ROOT"),
        }
    }
}

// == Account Field ==

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountField {
    Nonce,
    Balance,
    StorageRoot,
    CodeHash,
}

impl AccountField {
    pub fn variants() -> Vec<String> {
        vec![
            "NONCE".to_string(),
            "BALANCE".to_string(),
            "STORAGE_ROOT".to_string(),
            "CODE_HASH".to_string(),
        ]
    }
}

impl FromStr for AccountField {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "NONCE" => Ok(AccountField::Nonce),
            "BALANCE" => Ok(AccountField::Balance),
            "STORAGE_ROOT" => Ok(AccountField::StorageRoot),
            "CODE_HASH" => Ok(AccountField::CodeHash),
            _ => bail!("Unknown account field"),
        }
    }
}

impl DatalakeField for AccountField {
    fn from_index(index: u8) -> Result<Self> {
        match index {
            0 => Ok(AccountField::Nonce),
            1 => Ok(AccountField::Balance),
            2 => Ok(AccountField::StorageRoot),
            3 => Ok(AccountField::CodeHash),
            _ => bail!("Invalid account field index"),
        }
    }

    fn to_index(&self) -> u8 {
        match self {
            AccountField::Nonce => 0,
            AccountField::Balance => 1,
            AccountField::StorageRoot => 2,
            AccountField::CodeHash => 3,
        }
    }

    fn decode_field_from_rlp(&self, account_rlp: &[u8]) -> U256 {
        let decoded = <Account>::rlp_decode(account_rlp);
        match self {
            AccountField::Nonce => U256::from(decoded.nonce),
            AccountField::Balance => U256::from(decoded.balance),
            AccountField::StorageRoot => decoded.storage_root.into(),
            AccountField::CodeHash => decoded.code_hash.into(),
        }
    }
}

impl Display for AccountField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountField::Nonce => write!(f, "NONCE"),
            AccountField::Balance => write!(f, "BALANCE"),
            AccountField::StorageRoot => write!(f, "STORAGE_ROOT"),
            AccountField::CodeHash => write!(f, "CODE_HASH"),
        }
    }
}
