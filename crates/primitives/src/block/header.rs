use anyhow::{bail, Result};
use std::str::FromStr;

use alloy_primitives::{hex, Address, Bloom, Bytes, B256, U256};
use alloy_rlp::{Decodable, Encodable};
use reth_primitives::Header;
use serde::{Deserialize, Serialize};

pub struct BlockHeader(Header);

impl BlockHeader {
    pub fn new_from_header(value: Header) -> Self {
        BlockHeader(value)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new(
        parent_hash: B256,
        ommers_hash: B256,
        beneficiary: Address,
        state_root: B256,
        transactions_root: B256,
        receipts_root: B256,
        logs_bloom: Bloom,
        difficulty: U256,
        number: u64,
        gas_limit: u64,
        gas_used: u64,
        timestamp: u64,
        extra_data: Bytes,
        mix_hash: B256,
        nonce: u64,
        base_fee_per_gas: Option<u64>,
        withdrawals_root: Option<B256>,
        blob_gas_used: Option<u64>,
        excess_blob_gas: Option<u64>,
        parent_beacon_block_root: Option<B256>,
    ) -> Self {
        BlockHeader(Header {
            parent_hash,
            ommers_hash,
            beneficiary,
            state_root,
            transactions_root,
            receipts_root,
            logs_bloom,
            difficulty,
            number,
            gas_limit,
            gas_used,
            timestamp,
            extra_data,
            mix_hash,
            nonce,
            base_fee_per_gas,
            withdrawals_root,
            blob_gas_used,
            excess_blob_gas,
            parent_beacon_block_root,
        })
    }

    pub fn rlp_encode(&self) -> String {
        let mut buffer = Vec::<u8>::new();
        self.0.encode(&mut buffer);
        hex::encode(buffer)
    }

    pub fn rlp_decode(rlp: &str) -> Self {
        let decoded = <Header>::decode(&mut hex::decode(rlp).unwrap().as_slice()).unwrap();
        BlockHeader::new_from_header(decoded)
    }

    pub fn get_block_hash(&self) -> String {
        self.0.hash_slow().to_string()
    }
}

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
    pub fn from_index(index: u8) -> Option<Self> {
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

impl From<&BlockHeaderFromRpc> for BlockHeader {
    fn from(value: &BlockHeaderFromRpc) -> Self {
        Self(Header {
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
        })
    }
}

/// MMR metadata and proof returned from indexer
// example https://rs-indexer.api.herodotus.cloud/accumulators/mmr-meta-and-proof?deployed_on_chain=11155111&accumulates_chain=11155111&block_numbers=4952100&block_numbers=4952101&block_numbers=4952102&block_numbers=4952103&hashing_function=poseidon&contract_type=AGGREGATOR
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MMRFromIndexer {
    pub data: Vec<MMRDataFromIndexer>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MMRDataFromIndexer {
    pub meta: MMRMetaFromIndexer,
    pub proofs: Vec<MMRProofFromIndexer>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MMRMetaFromIndexer {
    pub mmr_id: u64,
    pub mmr_peaks: Vec<String>,
    pub mmr_root: String,
    pub mmr_size: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MMRProofFromIndexer {
    pub block_number: u64,
    pub element_hash: String,
    pub element_index: u64,
    pub rlp_block_header: String,
    pub siblings_hashes: Vec<String>,
}

#[cfg(test)]
mod tests {

    use std::str::FromStr;

    use alloy_primitives::{hex, Address, Bloom, Bytes, FixedBytes, U256};
    use alloy_rlp::Decodable;
    use reth_primitives::Header;

    #[test]
    pub fn test_rlp() {
        let rlp_hex ="f90266a045adb684cb5458019c496206c1383894c360fe969a1028ba44955eadfa585cc5a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d4934794b636a68f834b4d75af9edc5fb0138bb4758ed293a01db2388923f7c78680b4a46bae725637013d74ad787ec5c861d3ade3df882d81a093586eb5f2781ded334a2a03d178f41dc06f271d7f1ff429e4da6ef42d12a773a0361590775fea7857cc048b9324c03e96f287199803ce1440ff1e12c5c6008049b901000420000a200308000025201005a30400008962800402185dc600144280040082221400010101200458002b0d88008028004206808408400402108f0812246200240a204365100109051c082a020081204200001060440090044044448100082100028001060640c011401a802000090331000408243804009402201240802082820403801141050a4a00208283202050000f10058894008000411050512800220a200000042275800280894080000202460040030000408001ce00282400000002a8c24210000200014a30040015020b04800020608800000850440240c06100011002000000200988001800000880128a050400329081c144080a040800000480839eb0f68401c9c380836f9a8e8465aa87809f496c6c756d696e61746520446d6f63726174697a6520447374726962757465a0c653e1c1cee990147f4439776cc3ead6f175e081998c33c93da41653112e89ce8800000000000000000da039db3f9d1fe0756e5aef4e2f0241ad957e999e49c981809c018425d0080f6cd2830400008405320000a0713ce910d12e99ba96492ff2f6411d4e0a3e567ab419e92e60cf5fc4aa74db7a".to_string();
        let rlp = hex::decode(rlp_hex).unwrap();
        let decoded = <Header as Decodable>::decode(&mut rlp.as_slice()).unwrap();
        let expected_header = Header {
        parent_hash:FixedBytes::from_str( "0x45adb684cb5458019c496206c1383894c360fe969a1028ba44955eadfa585cc5").unwrap(),
        ommers_hash: FixedBytes::from_str( "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347").unwrap(),
        beneficiary: Address::from_str("0xb636a68f834b4d75af9edc5fb0138bb4758ed293").unwrap(),
        state_root:FixedBytes::from_str( "0x1db2388923f7c78680b4a46bae725637013d74ad787ec5c861d3ade3df882d81"
    ).unwrap(),
        transactions_root: FixedBytes::from_str("0x93586eb5f2781ded334a2a03d178f41dc06f271d7f1ff429e4da6ef42d12a773"
).unwrap(),
        receipts_root: FixedBytes::from_str("0x361590775fea7857cc048b9324c03e96f287199803ce1440ff1e12c5c6008049"
).unwrap(),
        logs_bloom:Bloom::from_str("0x0420000a200308000025201005a30400008962800402185dc600144280040082221400010101200458002b0d88008028004206808408400402108f0812246200240a204365100109051c082a020081204200001060440090044044448100082100028001060640c011401a802000090331000408243804009402201240802082820403801141050a4a00208283202050000f10058894008000411050512800220a200000042275800280894080000202460040030000408001ce00282400000002a8c24210000200014a30040015020b04800020608800000850440240c06100011002000000200988001800000880128a050400329081c144080a0408000004").unwrap(),
        difficulty: U256::from(0x0),
        number: 0x9eb0f6u64,
        gas_limit: 0x1c9c380u64,
        gas_used: 0x6f9a8eu64,
        timestamp: 0x65aa8780u64,
        extra_data: Bytes::from_str("0x496c6c756d696e61746520446d6f63726174697a6520447374726962757465").unwrap(),
        mix_hash: FixedBytes::from_str("0xc653e1c1cee990147f4439776cc3ead6f175e081998c33c93da41653112e89ce").unwrap(),
        nonce:0x0u64,
        base_fee_per_gas: Some(13),
        withdrawals_root: Some(FixedBytes::from_str("0x39db3f9d1fe0756e5aef4e2f0241ad957e999e49c981809c018425d0080f6cd2").unwrap()),
        blob_gas_used: Some(0x40000u64),
        excess_blob_gas: Some(0x5320000u64),
        parent_beacon_block_root: Some(FixedBytes::from_str("0x713ce910d12e99ba96492ff2f6411d4e0a3e567ab419e92e60cf5fc4aa74db7a").unwrap()),
    };

        assert_eq!(decoded, expected_header);
    }
}