use std::str::FromStr;

use alloy_primitives::{hex, Address, Bloom, Bytes, B256, U256};
use alloy_rlp::Decodable;
use reth_primitives::Header;
use serde::{Deserialize, Serialize};

use crate::datalake::base::DataPoint;

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

pub fn decode_header_field(header_rlp: &str, field: HeaderField) -> DataPoint {
    let decoded =
        <Header as Decodable>::decode(&mut hex::decode(header_rlp).unwrap().as_slice()).unwrap();

    match field {
        HeaderField::ParentHash => DataPoint::Str(decoded.parent_hash.to_string()),
        HeaderField::OmmerHash => DataPoint::Str(decoded.ommers_hash.to_string()),
        HeaderField::Beneficiary => DataPoint::Str(decoded.beneficiary.to_string()),
        HeaderField::StateRoot => DataPoint::Str(decoded.state_root.to_string()),
        HeaderField::TransactionsRoot => DataPoint::Str(decoded.transactions_root.to_string()),
        HeaderField::ReceiptsRoot => DataPoint::Str(decoded.receipts_root.to_string()),
        HeaderField::LogsBloom => DataPoint::Str(decoded.logs_bloom.to_string()),
        HeaderField::Difficulty => {
            DataPoint::Int(u64::from_str(&decoded.difficulty.to_string()).unwrap())
        }
        HeaderField::Number => DataPoint::Int(u64::from_str(&decoded.number.to_string()).unwrap()),
        HeaderField::GasLimit => {
            DataPoint::Int(u64::from_str(&decoded.gas_limit.to_string()).unwrap())
        }
        HeaderField::GasUsed => {
            DataPoint::Int(u64::from_str(&decoded.gas_used.to_string()).unwrap())
        }
        HeaderField::Timestamp => {
            DataPoint::Int(u64::from_str(&decoded.timestamp.to_string()).unwrap())
        }
        HeaderField::ExtraData => DataPoint::Str(decoded.extra_data.to_string()),
        HeaderField::MixHash => DataPoint::Str(decoded.mix_hash.to_string()),
        HeaderField::Nonce => DataPoint::Int(u64::from_str(&decoded.nonce.to_string()).unwrap()),
        HeaderField::BaseFeePerGas => {
            DataPoint::Int(u64::from_str(&decoded.base_fee_per_gas.unwrap().to_string()).unwrap())
        }
        HeaderField::WithdrawalsRoot => {
            DataPoint::Str(decoded.withdrawals_root.unwrap().to_string())
        }
        HeaderField::BlobGasUsed => {
            DataPoint::Int(u64::from_str(&decoded.blob_gas_used.unwrap().to_string()).unwrap())
        }
        HeaderField::ExcessBlobGas => {
            DataPoint::Int(u64::from_str(&decoded.excess_blob_gas.unwrap().to_string()).unwrap())
        }
        HeaderField::ParentBeaconBlockRoot => {
            DataPoint::Str(decoded.parent_beacon_block_root.unwrap().to_string())
        }
    }
}

/// Block header returned from RPC
/// https://ethereum.org/en/developers/docs/apis/json-rpc#eth_getblockbynumber
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BlockHeaderFromRpc {
    pub base_fee_per_gas: Option<String>,
    pub blob_gas_used: Option<String>,
    pub difficulty: String,
    pub excess_blob_gas: Option<String>,
    pub extra_data: String,
    pub gas_limit: String,
    pub gas_used: String,
    pub hash: String,
    pub logs_bloom: String,
    pub miner: String,
    pub mix_hash: String,
    pub nonce: String,
    pub number: String,
    pub parent_beacon_block_root: Option<String>,
    pub parent_hash: String,
    pub receipts_root: String,
    pub sha3_uncles: String,
    pub size: String,
    pub state_root: String,
    pub timestamp: String,
    pub total_difficulty: String,
    pub transactions_root: String,
    pub withdrawals_root: Option<String>,
}

impl BlockHeaderFromRpc {
    pub fn get_block_hash(&self) -> String {
        self.hash.clone()
    }
}

impl From<&BlockHeaderFromRpc> for Header {
    fn from(value: &BlockHeaderFromRpc) -> Self {
        Self {
            parent_hash: B256::from_str(&value.parent_hash).expect("Invalid hex string"),
            ommers_hash: B256::from_str(&value.sha3_uncles).expect("Invalid hex string"),
            beneficiary: Address::from_str(&value.miner).expect("Invalid hex string"),
            state_root: B256::from_str(&value.state_root).expect("Invalid hex string"),
            transactions_root: B256::from_str(&value.transactions_root)
                .expect("Invalid hex string"),
            receipts_root: B256::from_str(&value.receipts_root).expect("Invalid hex string"),
            logs_bloom: Bloom::from_str(&value.logs_bloom).expect("Invalid hex string"),
            difficulty: U256::from_str_radix(&value.difficulty[2..], 16)
                .expect("Invalid hex string"),
            number: u64::from_str_radix(&value.number[2..], 16).expect("Invalid hex string"),
            gas_limit: u64::from_str_radix(&value.gas_limit[2..], 16).expect("Invalid hex string"),
            gas_used: u64::from_str_radix(&value.gas_used[2..], 16).expect("Invalid hex string"),
            timestamp: u64::from_str_radix(&value.timestamp[2..], 16).expect("Invalid hex string"),
            extra_data: Bytes::from_str(&value.extra_data).expect("Invalid hex string"),
            mix_hash: B256::from_str(&value.mix_hash).expect("Invalid hex string"),
            nonce: u64::from_str_radix(&value.nonce[2..], 16).expect("Invalid hex string"),
            base_fee_per_gas: value
                .base_fee_per_gas
                .clone()
                .map(|x| u64::from_str_radix(&x[2..], 16).expect("Invalid hex string")),
            withdrawals_root: value
                .withdrawals_root
                .clone()
                .map(|x| B256::from_str(&x).expect("Invalid hex string")),
            blob_gas_used: value
                .blob_gas_used
                .clone()
                .map(|x| u64::from_str_radix(&x[2..], 16).expect("Invalid hex string")),
            excess_blob_gas: value
                .excess_blob_gas
                .clone()
                .map(|x| u64::from_str_radix(&x[2..], 16).expect("Invalid hex string")),
            parent_beacon_block_root: value
                .parent_beacon_block_root
                .clone()
                .map(|x| B256::from_str(&x).expect("Invalid hex string")),
        }
    }
}
