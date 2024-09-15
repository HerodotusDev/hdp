use std::str::FromStr;

use alloy::{
    hex,
    primitives::{keccak256, Address, BlockNumber, Bloom, Bytes, B256, B64, U256},
};
use alloy_rlp::{length_of_length, BufMut, Decodable, Encodable};
use serde::{Deserialize, Serialize};

// =============================================================================
// Header (credit: https://github.com/paradigmxyz/reth/blob/main/crates/primitives/src/header.rs#L133)
// Orignally had dependnecy on `reth_primitives` crate, but it was removed to publish in crates.io
// =============================================================================
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Header {
    /// The Keccak 256-bit hash of the parent
    /// block’s header, in its entirety; formally Hp.
    pub parent_hash: B256,
    /// The Keccak 256-bit hash of the ommers list portion of this block; formally Ho.
    pub ommers_hash: B256,
    /// The 160-bit address to which all fees collected from the successful mining of this block
    /// be transferred; formally Hc.
    pub beneficiary: Address,
    /// The Keccak 256-bit hash of the root node of the state trie, after all transactions are
    /// executed and finalisations applied; formally Hr.
    pub state_root: B256,
    /// The Keccak 256-bit hash of the root node of the trie structure populated with each
    /// transaction in the transactions list portion of the block; formally Ht.
    pub transactions_root: B256,
    /// The Keccak 256-bit hash of the root node of the trie structure populated with the receipts
    /// of each transaction in the transactions list portion of the block; formally He.
    pub receipts_root: B256,
    /// The Keccak 256-bit hash of the withdrawals list portion of this block.
    /// <https://eips.ethereum.org/EIPS/eip-4895>
    pub withdrawals_root: Option<B256>,
    /// The Bloom filter composed from indexable information (logger address and log topics)
    /// contained in each log entry from the receipt of each transaction in the transactions list;
    /// formally Hb.
    pub logs_bloom: Bloom,
    /// A scalar value corresponding to the difficulty level of this block. This can be calculated
    /// from the previous block’s difficulty level and the timestamp; formally Hd.
    pub difficulty: U256,
    /// A scalar value equal to the number of ancestor blocks. The genesis block has a number of
    /// zero; formally Hi.
    pub number: BlockNumber,
    /// A scalar value equal to the current limit of gas expenditure per block; formally Hl.
    pub gas_limit: u64,
    /// A scalar value equal to the total gas used in transactions in this block; formally Hg.
    pub gas_used: u64,
    /// A scalar value equal to the reasonable output of Unix’s time() at this block’s inception;
    /// formally Hs.
    pub timestamp: u64,
    /// A 256-bit hash which, combined with the
    /// nonce, proves that a sufficient amount of computation has been carried out on this block;
    /// formally Hm.
    pub mix_hash: B256,
    /// A 64-bit value which, combined with the mixhash, proves that a sufficient amount of
    /// computation has been carried out on this block; formally Hn.
    pub nonce: u64,
    /// A scalar representing EIP1559 base fee which can move up or down each block according
    /// to a formula which is a function of gas used in parent block and gas target
    /// (block gas limit divided by elasticity multiplier) of parent block.
    /// The algorithm results in the base fee per gas increasing when blocks are
    /// above the gas target, and decreasing when blocks are below the gas target. The base fee per
    /// gas is burned.
    pub base_fee_per_gas: Option<u64>,
    /// The total amount of blob gas consumed by the transactions within the block, added in
    /// EIP-4844.
    pub blob_gas_used: Option<u64>,
    /// A running total of blob gas consumed in excess of the target, prior to the block. Blocks
    /// with above-target blob gas consumption increase this value, blocks with below-target blob
    /// gas consumption decrease it (bounded at 0). This was added in EIP-4844.
    pub excess_blob_gas: Option<u64>,
    /// The hash of the parent beacon block's root is included in execution blocks, as proposed by
    /// EIP-4788.
    ///
    /// This enables trust-minimized access to consensus state, supporting staking pools, bridges,
    /// and more.
    ///
    /// The beacon roots contract handles root storage, enhancing Ethereum's functionalities.
    pub parent_beacon_block_root: Option<B256>,
    /// An arbitrary byte array containing data relevant to this block. This must be 32 bytes or
    /// fewer; formally Hx.
    pub extra_data: Bytes,
}

impl Header {
    fn header_payload_length(&self) -> usize {
        let mut length = 0;
        length += self.parent_hash.length(); // Hash of the previous block.
        length += self.ommers_hash.length(); // Hash of uncle blocks.
        length += self.beneficiary.length(); // Address that receives rewards.
        length += self.state_root.length(); // Root hash of the state object.
        length += self.transactions_root.length(); // Root hash of transactions in the block.
        length += self.receipts_root.length(); // Hash of transaction receipts.
        length += self.logs_bloom.length(); // Data structure containing event logs.
        length += self.difficulty.length(); // Difficulty value of the block.
        length += U256::from(self.number).length(); // Block number.
        length += U256::from(self.gas_limit).length(); // Maximum gas allowed.
        length += U256::from(self.gas_used).length(); // Actual gas used.
        length += self.timestamp.length(); // Block timestamp.
        length += self.extra_data.length(); // Additional arbitrary data.
        length += self.mix_hash.length(); // Hash used for mining.
        length += B64::new(self.nonce.to_be_bytes()).length(); // Nonce for mining.

        if let Some(base_fee) = self.base_fee_per_gas {
            // Adding base fee length if it exists.
            length += U256::from(base_fee).length();
        }

        if let Some(root) = self.withdrawals_root {
            // Adding withdrawals_root length if it exists.
            length += root.length();
        }

        if let Some(blob_gas_used) = self.blob_gas_used {
            // Adding blob_gas_used length if it exists.
            length += U256::from(blob_gas_used).length();
        }

        if let Some(excess_blob_gas) = self.excess_blob_gas {
            // Adding excess_blob_gas length if it exists.
            length += U256::from(excess_blob_gas).length();
        }

        if let Some(parent_beacon_block_root) = self.parent_beacon_block_root {
            length += parent_beacon_block_root.length();
        }

        length
    }

    /// Heavy function that will calculate hash of data and will *not* save the change to metadata.
    pub fn hash_slow(&self) -> B256 {
        keccak256(alloy_rlp::encode(self))
    }
}

impl Encodable for Header {
    fn encode(&self, out: &mut dyn BufMut) {
        // Create a header indicating the encoded content is a list with the payload length computed
        // from the header's payload calculation function.
        let list_header = alloy_rlp::Header {
            list: true,
            payload_length: self.header_payload_length(),
        };
        list_header.encode(out);

        // Encode each header field sequentially
        self.parent_hash.encode(out); // Encode parent hash.
        self.ommers_hash.encode(out); // Encode ommer's hash.
        self.beneficiary.encode(out); // Encode beneficiary.
        self.state_root.encode(out); // Encode state root.
        self.transactions_root.encode(out); // Encode transactions root.
        self.receipts_root.encode(out); // Encode receipts root.
        self.logs_bloom.encode(out); // Encode logs bloom.
        self.difficulty.encode(out); // Encode difficulty.
        U256::from(self.number).encode(out); // Encode block number.
        U256::from(self.gas_limit).encode(out); // Encode gas limit.
        U256::from(self.gas_used).encode(out); // Encode gas used.
        self.timestamp.encode(out); // Encode timestamp.
        self.extra_data.encode(out); // Encode extra data.
        self.mix_hash.encode(out); // Encode mix hash.
        B64::new(self.nonce.to_be_bytes()).encode(out); // Encode nonce.

        // Encode base fee. Put empty list if base fee is missing,
        // but withdrawals root is present.
        if let Some(ref base_fee) = self.base_fee_per_gas {
            U256::from(*base_fee).encode(out);
        }

        // Encode withdrawals root. Put empty string if withdrawals root is missing,
        // but blob gas used is present.
        if let Some(ref root) = self.withdrawals_root {
            root.encode(out);
        }

        // Encode blob gas used. Put empty list if blob gas used is missing,
        // but excess blob gas is present.
        if let Some(ref blob_gas_used) = self.blob_gas_used {
            U256::from(*blob_gas_used).encode(out);
        }

        // Encode excess blob gas. Put empty list if excess blob gas is missing,
        // but parent beacon block root is present.
        if let Some(ref excess_blob_gas) = self.excess_blob_gas {
            U256::from(*excess_blob_gas).encode(out);
        }

        // Encode parent beacon block root. If new fields are added, the above pattern will need to
        // be repeated and placeholders added. Otherwise, it's impossible to tell _which_
        // fields are missing. This is mainly relevant for contrived cases where a header is
        // created at random, for example:
        //  * A header is created with a withdrawals root, but no base fee. Shanghai blocks are
        //    post-London, so this is technically not valid. However, a tool like proptest would
        //    generate a block like this.
        if let Some(ref parent_beacon_block_root) = self.parent_beacon_block_root {
            parent_beacon_block_root.encode(out);
        }
    }

    fn length(&self) -> usize {
        let mut length = 0;
        length += self.header_payload_length();
        length += length_of_length(length);
        length
    }
}

impl Decodable for Header {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let rlp_head = alloy_rlp::Header::decode(buf)?;
        if !rlp_head.list {
            return Err(alloy_rlp::Error::UnexpectedString);
        }
        let started_len = buf.len();
        let mut this = Self {
            parent_hash: Decodable::decode(buf)?,
            ommers_hash: Decodable::decode(buf)?,
            beneficiary: Decodable::decode(buf)?,
            state_root: Decodable::decode(buf)?,
            transactions_root: Decodable::decode(buf)?,
            receipts_root: Decodable::decode(buf)?,
            logs_bloom: Decodable::decode(buf)?,
            difficulty: Decodable::decode(buf)?,
            number: u64::decode(buf)?,
            gas_limit: u64::decode(buf)?,
            gas_used: u64::decode(buf)?,
            timestamp: Decodable::decode(buf)?,
            extra_data: Decodable::decode(buf)?,
            mix_hash: Decodable::decode(buf)?,
            nonce: u64::from_be_bytes(B64::decode(buf)?.0),
            base_fee_per_gas: None,
            withdrawals_root: None,
            blob_gas_used: None,
            excess_blob_gas: None,
            parent_beacon_block_root: None,
        };
        if started_len - buf.len() < rlp_head.payload_length {
            this.base_fee_per_gas = Some(u64::decode(buf)?);
        }

        // Withdrawals root for post-shanghai headers
        if started_len - buf.len() < rlp_head.payload_length {
            this.withdrawals_root = Some(Decodable::decode(buf)?);
        }

        // Blob gas used and excess blob gas for post-cancun headers
        if started_len - buf.len() < rlp_head.payload_length {
            this.blob_gas_used = Some(u64::decode(buf)?);
        }

        if started_len - buf.len() < rlp_head.payload_length {
            this.excess_blob_gas = Some(u64::decode(buf)?);
        }

        // Decode parent beacon block root. If new fields are added, the above pattern will need to
        // be repeated and placeholders decoded. Otherwise, it's impossible to tell _which_
        // fields are missing. This is mainly relevant for contrived cases where a header is
        // created at random, for example:
        //  * A header is created with a withdrawals root, but no base fee. Shanghai blocks are
        //    post-London, so this is technically not valid. However, a tool like proptest would
        //    generate a block like this.
        if started_len - buf.len() < rlp_head.payload_length {
            this.parent_beacon_block_root = Some(B256::decode(buf)?);
        }

        let consumed = started_len - buf.len();
        if consumed != rlp_head.payload_length {
            return Err(alloy_rlp::Error::ListLengthMismatch {
                expected: rlp_head.payload_length,
                got: consumed,
            });
        }
        Ok(this)
    }
}

// =============================================================================

impl Header {
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
        Header {
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
        }
    }

    pub fn rlp_encode(&self) -> Vec<u8> {
        let mut buffer = Vec::<u8>::new();
        self.encode(&mut buffer);
        buffer
    }

    pub fn rlp_decode(mut rlp: &[u8]) -> Self {
        <Header>::decode(&mut rlp).unwrap()
    }

    pub fn get_block_hash(&self) -> String {
        self.hash_slow().to_string()
    }
}

/// Block header returned from RPC
/// <https://ethereum.org/en/developers/docs/apis/json-rpc#eth_getblockbynumber>
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

impl From<BlockHeaderFromRpc> for Header {
    fn from(value: BlockHeaderFromRpc) -> Self {
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
                .map(|x| u64::from_str_radix(&x[2..], 16).expect("Invalid hex string")),
            withdrawals_root: value
                .withdrawals_root
                .map(|x| B256::from_str(&x).expect("Invalid hex string")),
            blob_gas_used: value
                .blob_gas_used
                .map(|x| u64::from_str_radix(&x[2..], 16).expect("Invalid hex string")),
            excess_blob_gas: value
                .excess_blob_gas
                .map(|x| u64::from_str_radix(&x[2..], 16).expect("Invalid hex string")),
            parent_beacon_block_root: value
                .parent_beacon_block_root
                .map(|x| B256::from_str(&x).expect("Invalid hex string")),
        }
    }
}

// THIS ENDPOINT IS DEPRECATED
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
    pub mmr_id: String,
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

/// MMR metadata and proof returned from indexer
// example https://rs-indexer.api.herodotus.cloud/accumulators/proofs
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MMRFromNewIndexer {
    pub data: Vec<MMRDataFromNewIndexer>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MMRDataFromNewIndexer {
    pub meta: MMRMetaFromNewIndexer,
    pub proofs: Vec<MMRProofFromNewIndexer>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MMRMetaFromNewIndexer {
    pub mmr_id: String,
    pub mmr_peaks: Vec<String>,
    pub mmr_root: String,
    pub mmr_size: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RlpBlockHeader {
    #[serde(rename = "String")]
    pub value: String,
}

impl From<RlpBlockHeader> for Bytes {
    fn from(rlp_block_header: RlpBlockHeader) -> Self {
        Bytes::from(hex::decode(rlp_block_header.value).expect("Cannot decode RLP block header"))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MMRProofFromNewIndexer {
    pub block_number: u64,
    pub element_hash: String,
    pub element_index: u64,
    #[serde(rename = "rlp_block_header")]
    pub rlp_block_header: RlpBlockHeader,
    pub siblings_hashes: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    use alloy::primitives::{hex, Address, Bloom, Bytes, FixedBytes, U256};
    use alloy_rlp::Decodable;

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
