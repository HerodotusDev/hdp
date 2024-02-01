use std::str::FromStr;

use alloy_primitives::hex::{encode, FromHex};
use alloy_rlp::{Decodable, Encodable, RlpDecodable, RlpEncodable};

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
            _ => None,
        }
    }

    pub fn to_index(&self) -> Option<usize> {
        match self {
            HeaderField::ParentHash => Some(0),
            HeaderField::OmmerHash => Some(1),
            HeaderField::Beneficiary => Some(2),
            HeaderField::StateRoot => Some(3),
            HeaderField::TransactionsRoot => Some(4),
            HeaderField::ReceiptsRoot => Some(5),
            HeaderField::LogsBloom => Some(6),
            HeaderField::Difficulty => Some(7),
            HeaderField::Number => Some(8),
            HeaderField::GasLimit => Some(9),
            HeaderField::GasUsed => Some(10),
            HeaderField::Timestamp => Some(11),
            HeaderField::ExtraData => Some(12),
            HeaderField::MixHash => Some(13),
            HeaderField::Nonce => Some(14),
            HeaderField::BaseFeePerGas => Some(15),
            HeaderField::WithdrawalsRoot => Some(16),
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
            _ => Err(()),
        }
    }
}

#[derive(Debug, RlpDecodable, RlpEncodable, PartialEq)]
#[rlp(trailing)]
pub struct BlockHeaderShanghai {
    pub parent_hash: String,
    pub uncle_hash: String,
    pub coinbase: String,
    pub state_root: String,
    pub transactions_root: String,
    pub receipts_root: String,
    pub logs_bloom: String,
    pub difficulty: u64,
    pub number: u64,
    pub gas_limit: u64,
    pub gas_used: u64,
    pub timestamp: u64,
    pub extra_data: String,
    pub mix_hash: String,
    pub nonce: String,
    pub base_fee_per_gas: Option<u64>,
    pub withdrawals_root: Option<String>,
}

impl BlockHeaderShanghai {
    pub fn from_rlp_hexstring(rlp_hexstring: &str) -> Self {
        let buffer = Vec::<u8>::from_hex(rlp_hexstring).unwrap();
        let rlp_decoded_header = BlockHeaderShanghai::decode(&mut buffer.as_slice()).unwrap();
        rlp_decoded_header
    }

    pub fn to_rlp_hexstring(&self) -> String {
        let mut buffer = Vec::<u8>::new();
        self.encode(&mut buffer);
        encode(buffer)
    }
}

pub fn decode_header_field(header_rlp: &str, field: HeaderField) -> String {
    let decoded = BlockHeaderShanghai::from_rlp_hexstring(header_rlp);

    match field {
        HeaderField::ParentHash => decoded.parent_hash,
        HeaderField::OmmerHash => decoded.uncle_hash,
        HeaderField::Beneficiary => decoded.coinbase,
        HeaderField::StateRoot => decoded.state_root,
        HeaderField::TransactionsRoot => decoded.transactions_root,
        HeaderField::ReceiptsRoot => decoded.receipts_root,
        HeaderField::LogsBloom => decoded.logs_bloom,
        HeaderField::Difficulty => decoded.difficulty.to_string(),
        HeaderField::Number => decoded.number.to_string(),
        HeaderField::GasLimit => decoded.gas_limit.to_string(),
        HeaderField::GasUsed => decoded.gas_used.to_string(),
        HeaderField::Timestamp => decoded.timestamp.to_string(),
        HeaderField::ExtraData => decoded.extra_data,
        HeaderField::MixHash => decoded.mix_hash,
        HeaderField::Nonce => decoded.nonce,
        HeaderField::BaseFeePerGas => decoded.base_fee_per_gas.unwrap_or(0).to_string(),
        HeaderField::WithdrawalsRoot => decoded.withdrawals_root.unwrap_or("".to_string()),
    }
}
