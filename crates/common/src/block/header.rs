use std::str::FromStr;

use alloy_primitives::hex;
use alloy_rlp::Decodable;
use reth_primitives::Header;

#[derive(Debug)]
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
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(HeaderField::ParentHash),
            1 => Some(HeaderField::OmmerHash),
            2 => Some(HeaderField::Beneficiary),
            3 => Some(HeaderField::StateRoot),
            4 => Some(HeaderField::TransactionsRoot),
            5 => Some(HeaderField::ReceiptsRoot),
            6 => Some(HeaderField::LogsBloom),
            7 => Some(HeaderField::Difficulty),
            8 => Some(HeaderField::Number),
            9 => Some(HeaderField::GasLimit),
            10 => Some(HeaderField::GasUsed),
            11 => Some(HeaderField::Timestamp),
            12 => Some(HeaderField::ExtraData),
            13 => Some(HeaderField::MixHash),
            14 => Some(HeaderField::Nonce),
            15 => Some(HeaderField::BaseFeePerGas),
            16 => Some(HeaderField::WithdrawalsRoot),
            17 => Some(HeaderField::BlobGasUsed),
            18 => Some(HeaderField::ExcessBlobGas),
            19 => Some(HeaderField::ParentBeaconBlockRoot),
            _ => None,
        }
    }

    pub fn to_index(&self) -> usize {
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
}

impl FromStr for HeaderField {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
            _ => Err(()),
        }
    }
}

pub fn decode_header_field(header_rlp: &str, field: HeaderField) -> String {
    let decoded =
        <Header as Decodable>::decode(&mut hex::decode(header_rlp).unwrap().as_slice()).unwrap();

    match field {
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
        HeaderField::ParentBeaconBlockRoot => decoded.parent_beacon_block_root.unwrap().to_string(),
    }
}
