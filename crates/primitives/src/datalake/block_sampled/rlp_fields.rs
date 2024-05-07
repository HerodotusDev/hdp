//! This module defines the fields that can be queried from a block or account.
//! The fields are defined as enums, and can be converted to and from their string representation.
//! It is meant to be used in the `BlockSampled` struct, which is used to query fields from a block or account.

use std::str::FromStr;

use anyhow::{bail, Result};

use crate::{
    block::{account::Account, header::Header},
    datalake::DatalakeField,
};

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

    fn decode_field_from_rlp(&self, header_rlp: &str) -> String {
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

impl ToString for HeaderField {
    fn to_string(&self) -> String {
        match self {
            HeaderField::ParentHash => "PARENT_HASH".to_string(),
            HeaderField::OmmerHash => "OMMERS_HASH".to_string(),
            HeaderField::Beneficiary => "BENEFICIARY".to_string(),
            HeaderField::StateRoot => "STATE_ROOT".to_string(),
            HeaderField::TransactionsRoot => "TRANSACTIONS_ROOT".to_string(),
            HeaderField::ReceiptsRoot => "RECEIPTS_ROOT".to_string(),
            HeaderField::LogsBloom => "LOGS_BLOOM".to_string(),
            HeaderField::Difficulty => "DIFFICULTY".to_string(),
            HeaderField::Number => "NUMBER".to_string(),
            HeaderField::GasLimit => "GAS_LIMIT".to_string(),
            HeaderField::GasUsed => "GAS_USED".to_string(),
            HeaderField::Timestamp => "TIMESTAMP".to_string(),
            HeaderField::ExtraData => "EXTRA_DATA".to_string(),
            HeaderField::MixHash => "MIX_HASH".to_string(),
            HeaderField::Nonce => "NONCE".to_string(),
            HeaderField::BaseFeePerGas => "BASE_FEE_PER_GAS".to_string(),
            HeaderField::WithdrawalsRoot => "WITHDRAWALS_ROOT".to_string(),
            HeaderField::BlobGasUsed => "BLOB_GAS_USED".to_string(),
            HeaderField::ExcessBlobGas => "EXCESS_BLOB_GAS".to_string(),
            HeaderField::ParentBeaconBlockRoot => "PARENT_BEACON_BLOCK_ROOT".to_string(),
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

    fn decode_field_from_rlp(&self, account_rlp: &str) -> String {
        let decoded = <Account>::rlp_decode(account_rlp);
        match self {
            AccountField::Nonce => decoded.nonce.to_string(),
            AccountField::Balance => decoded.balance.to_string(),
            AccountField::StorageRoot => decoded.storage_root.to_string(),
            AccountField::CodeHash => decoded.code_hash.to_string(),
        }
    }
}

impl ToString for AccountField {
    fn to_string(&self) -> String {
        match self {
            AccountField::Nonce => "NONCE".to_string(),
            AccountField::Balance => "BALANCE".to_string(),
            AccountField::StorageRoot => "STORAGE_ROOT".to_string(),
            AccountField::CodeHash => "CODE_HASH".to_string(),
        }
    }
}
