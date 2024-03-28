//! This module defines the fields that can be queried from a block or account.
//! The fields are defined as enums, and can be converted to and from their string representation.
//! It is meant to be used in the `BlockSampled` struct, which is used to query fields from a block or account.

use std::str::FromStr;

use anyhow::{bail, Result};

use crate::block::{account::Account, header::Header};

#[derive(Debug, Clone, PartialEq)]
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
    pub fn from_index(index: u8) -> Result<Self> {
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

    pub fn to_index(&self) -> u8 {
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

    pub fn as_str(&self) -> &'static str {
        match self {
            HeaderField::ParentHash => "PARENT_HASH",
            HeaderField::OmmerHash => "OMMERS_HASH",
            HeaderField::Beneficiary => "BENEFICIARY",
            HeaderField::StateRoot => "STATE_ROOT",
            HeaderField::TransactionsRoot => "TRANSACTIONS_ROOT",
            HeaderField::ReceiptsRoot => "RECEIPTS_ROOT",
            HeaderField::LogsBloom => "LOGS_BLOOM",
            HeaderField::Difficulty => "DIFFICULTY",
            HeaderField::Number => "NUMBER",
            HeaderField::GasLimit => "GAS_LIMIT",
            HeaderField::GasUsed => "GAS_USED",
            HeaderField::Timestamp => "TIMESTAMP",
            HeaderField::ExtraData => "EXTRA_DATA",
            HeaderField::MixHash => "MIX_HASH",
            HeaderField::Nonce => "NONCE",
            HeaderField::BaseFeePerGas => "BASE_FEE_PER_GAS",
            HeaderField::WithdrawalsRoot => "WITHDRAWALS_ROOT",
            HeaderField::BlobGasUsed => "BLOB_GAS_USED",
            HeaderField::ExcessBlobGas => "EXCESS_BLOB_GAS",
            HeaderField::ParentBeaconBlockRoot => "PARENT_BEACON_BLOCK_ROOT",
        }
    }

    pub fn decode_rlp(&self, header_rlp: &str) -> String {
        let decoded = <Header>::rlp_decode(header_rlp);

        match self {
            HeaderField::ParentHash => decoded.parent_hash.to_string(),
            HeaderField::OmmerHash => decoded.ommers_hash.to_string(),
            HeaderField::Beneficiary => decoded.beneficiary.to_string(),
            HeaderField::StateRoot => decoded.state_root.to_string(),
            HeaderField::TransactionsRoot => decoded.transactions_root.to_string(),
            HeaderField::ReceiptsRoot => decoded.receipts_root.to_string(),
            HeaderField::LogsBloom => decoded.logs_bloom.to_string(),
            HeaderField::Difficulty => decoded.difficulty.to_string(),
            HeaderField::Number => decoded.number.to_string(),
            HeaderField::GasLimit => decoded.gas_limit.to_string(),
            HeaderField::GasUsed => decoded.gas_used.to_string(),
            HeaderField::Timestamp => decoded.timestamp.to_string(),
            HeaderField::ExtraData => decoded.extra_data.to_string(),
            HeaderField::MixHash => decoded.mix_hash.to_string(),
            HeaderField::Nonce => decoded.nonce.to_string(),
            HeaderField::BaseFeePerGas => decoded.base_fee_per_gas.unwrap().to_string(),
            HeaderField::WithdrawalsRoot => decoded.withdrawals_root.unwrap().to_string(),
            HeaderField::BlobGasUsed => decoded.blob_gas_used.unwrap().to_string(),
            HeaderField::ExcessBlobGas => decoded.excess_blob_gas.unwrap().to_string(),
            HeaderField::ParentBeaconBlockRoot => {
                decoded.parent_beacon_block_root.unwrap().to_string()
            }
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

// == Account Field ==

#[derive(Debug, Clone, PartialEq)]
pub enum AccountField {
    Nonce,
    Balance,
    StorageRoot,
    CodeHash,
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

impl AccountField {
    pub fn from_index(index: u8) -> Result<Self> {
        match index {
            0 => Ok(AccountField::Nonce),
            1 => Ok(AccountField::Balance),
            2 => Ok(AccountField::StorageRoot),
            3 => Ok(AccountField::CodeHash),
            _ => bail!("Invalid account field index"),
        }
    }

    pub fn to_index(&self) -> u8 {
        match self {
            AccountField::Nonce => 0,
            AccountField::Balance => 1,
            AccountField::StorageRoot => 2,
            AccountField::CodeHash => 3,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AccountField::Nonce => "NONCE",
            AccountField::Balance => "BALANCE",
            AccountField::StorageRoot => "STORAGE_ROOT",
            AccountField::CodeHash => "CODE_HASH",
        }
    }

    pub fn decode_rlp(&self, account_rlp: &str) -> String {
        let decoded = <Account>::rlp_decode(account_rlp);
        match self {
            AccountField::Nonce => decoded.nonce.to_string(),
            AccountField::Balance => decoded.balance.to_string(),
            AccountField::StorageRoot => decoded.storage_root.to_string(),
            AccountField::CodeHash => decoded.code_hash.to_string(),
        }
    }
}
