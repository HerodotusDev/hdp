//! Provider keys for fetching data from memoizer and rpc.
//! Only used for context of Module Compiler
//!
//! TODO: need to sync with how bootloader will emit the keys

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    str::FromStr,
};

use crate::primitives::{
    task::datalake::{
        block_sampled::BlockSampledCollectionType, transactions::TransactionsCollectionType,
    },
    ChainId,
};
use alloy::primitives::{Address, BlockNumber, Keccak256, StorageKey, B256};
use serde::{Deserialize, Serialize};
use starknet_crypto::Felt;

/// This is keys that are categorized into different subsets of keys.
#[derive(Debug, Default)]
pub struct CategorizedFetchKeys {
    pub evm_headers: HashSet<EvmHeaderKey>,
    pub evm_accounts: HashSet<EvmAccountKey>,
    pub evm_storages: HashSet<EvmStorageKey>,
    pub evm_txs: HashSet<EvmBlockTxKey>,
    pub evm_receipts: HashSet<EvmBlockReceiptKey>,
    pub sn_headers: HashSet<StarknetHeaderKey>,
    pub sn_storages: HashSet<StarknetStorageKey>,
}

impl CategorizedFetchKeys {
    pub fn new(
        evm_headers: HashSet<EvmHeaderKey>,
        evm_accounts: HashSet<EvmAccountKey>,
        evm_storages: HashSet<EvmStorageKey>,
        evm_txs: HashSet<EvmBlockTxKey>,
        evm_receipts: HashSet<EvmBlockReceiptKey>,
        sn_headers: HashSet<StarknetHeaderKey>,
        sn_storages: HashSet<StarknetStorageKey>,
    ) -> Self {
        Self {
            evm_headers,
            evm_accounts,
            evm_storages,
            evm_txs,
            evm_receipts,
            sn_headers,
            sn_storages,
        }
    }
}

/// Categorize fetch keys
/// This is require to initiate multiple provider for different chain and fetch keys types
pub fn categorize_fetch_keys(
    fetch_keys: Vec<FetchKeyEnvelope>,
) -> Vec<(ChainId, CategorizedFetchKeys)> {
    let mut chain_id_map: HashMap<ChainId, CategorizedFetchKeys> = std::collections::HashMap::new();

    for key in fetch_keys {
        let chain_id = key.get_chain_id();
        let target_categorized_fetch_keys = chain_id_map.entry(chain_id).or_default();

        match key {
            FetchKeyEnvelope::EvmHeader(header_key) => {
                target_categorized_fetch_keys.evm_headers.insert(header_key);
            }
            FetchKeyEnvelope::EvmAccount(account_key) => {
                target_categorized_fetch_keys
                    .evm_headers
                    .insert(EvmHeaderKey::new(
                        account_key.chain_id,
                        account_key.block_number,
                    ));
                target_categorized_fetch_keys
                    .evm_accounts
                    .insert(account_key);
            }
            FetchKeyEnvelope::EvmStorage(storage_key) => {
                target_categorized_fetch_keys
                    .evm_headers
                    .insert(EvmHeaderKey::new(
                        storage_key.chain_id,
                        storage_key.block_number,
                    ));
                target_categorized_fetch_keys
                    .evm_storages
                    .insert(storage_key);
            }
            FetchKeyEnvelope::EvmTx(tx_key) => {
                target_categorized_fetch_keys
                    .evm_headers
                    .insert(EvmHeaderKey::new(tx_key.chain_id, tx_key.block_number));
                target_categorized_fetch_keys.evm_txs.insert(tx_key);
            }
            FetchKeyEnvelope::EvmTxReceipt(tx_receipt_key) => {
                target_categorized_fetch_keys
                    .evm_headers
                    .insert(EvmHeaderKey::new(
                        tx_receipt_key.chain_id,
                        tx_receipt_key.block_number,
                    ));
                target_categorized_fetch_keys
                    .evm_receipts
                    .insert(tx_receipt_key);
            }
            FetchKeyEnvelope::StarknetHeader(header_key) => {
                target_categorized_fetch_keys.sn_headers.insert(header_key);
            }
            FetchKeyEnvelope::StarknetStorage(storage_key) => {
                target_categorized_fetch_keys
                    .sn_headers
                    .insert(StarknetHeaderKey::new(
                        storage_key.chain_id,
                        storage_key.block_number,
                    ));
                target_categorized_fetch_keys
                    .sn_storages
                    .insert(storage_key);
            }
        }
    }
    chain_id_map.into_iter().collect()
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct StarknetHeaderKey {
    pub chain_id: ChainId,
    pub block_number: u64,
}

impl StarknetHeaderKey {
    pub fn new(chain_id: ChainId, block_number: u64) -> Self {
        Self {
            chain_id,
            block_number,
        }
    }
}

impl<'de> Deserialize<'de> for StarknetHeaderKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            chain_id: String,
            block_number: u64,
        }

        let helper = Helper::deserialize(deserializer)?;

        Ok(StarknetHeaderKey {
            chain_id: ChainId::from_hex_str(&helper.chain_id).expect("invalid deserialize"),
            block_number: helper.block_number,
        })
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct StarknetStorageKey {
    pub chain_id: ChainId,
    pub block_number: u64,
    pub address: Felt,
    pub key: Felt,
}

impl StarknetStorageKey {
    pub fn new(chain_id: ChainId, block_number: u64, address: Felt, key: Felt) -> Self {
        Self {
            chain_id,
            block_number,
            address,
            key,
        }
    }
}

impl<'de> Deserialize<'de> for StarknetStorageKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            chain_id: String,
            block_number: u64,
            address: Felt,
            key: Felt,
        }

        let helper = Helper::deserialize(deserializer)?;

        Ok(StarknetStorageKey {
            chain_id: ChainId::from_hex_str(&helper.chain_id).expect("invalid deserialize"),
            block_number: helper.block_number,
            address: helper.address,
            key: helper.key,
        })
    }
}

/// Key for fetching block header from provider.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct EvmHeaderKey {
    pub chain_id: ChainId,
    pub block_number: BlockNumber,
}

impl<'de> Deserialize<'de> for EvmHeaderKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            chain_id: String,
            block_number: BlockNumber,
        }

        let helper = Helper::deserialize(deserializer)?;

        Ok(EvmHeaderKey {
            chain_id: ChainId::from_hex_str(&helper.chain_id).expect("invalid deserialize"),
            block_number: helper.block_number,
        })
    }
}

impl EvmHeaderKey {
    pub fn new(chain_id: ChainId, block_number: BlockNumber) -> Self {
        Self {
            chain_id,
            block_number,
        }
    }

    pub fn hash_key(&self) -> B256 {
        let mut keccak = Keccak256::new();
        keccak.update([BlockSampledCollectionType::Header.to_u8()]);
        keccak.update(self.chain_id.to_be_bytes());
        keccak.update(self.block_number.to_be_bytes());
        keccak.finalize()
    }
}

/// Key for fetching account from provider.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct EvmAccountKey {
    pub chain_id: ChainId,
    pub block_number: BlockNumber,
    pub address: Address,
}

impl<'de> Deserialize<'de> for EvmAccountKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            chain_id: String,
            block_number: BlockNumber,
            address: Address,
        }

        let helper = Helper::deserialize(deserializer)?;

        Ok(EvmAccountKey {
            chain_id: ChainId::from_hex_str(&helper.chain_id).expect("invalid deserialize"),
            block_number: helper.block_number,
            address: helper.address,
        })
    }
}

impl EvmAccountKey {
    pub fn new(chain_id: ChainId, block_number: BlockNumber, address: Address) -> Self {
        Self {
            chain_id,
            block_number,
            address,
        }
    }

    pub fn hash_key(&self) -> B256 {
        let mut keccak = Keccak256::new();
        keccak.update([BlockSampledCollectionType::Account.to_u8()]);
        keccak.update(self.chain_id.to_be_bytes());
        keccak.update(self.block_number.to_be_bytes());
        keccak.update(self.address);
        keccak.finalize()
    }
}

/// Key for fetching storage value from provider.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct EvmStorageKey {
    pub chain_id: ChainId,
    pub block_number: BlockNumber,
    pub address: Address,
    pub key: StorageKey,
}

impl<'de> Deserialize<'de> for EvmStorageKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            chain_id: String,
            block_number: BlockNumber,
            address: Address,
            key: StorageKey,
        }

        let helper = Helper::deserialize(deserializer)?;

        Ok(EvmStorageKey {
            chain_id: ChainId::from_hex_str(&helper.chain_id).expect("invalid deserialize"),
            block_number: helper.block_number,
            address: helper.address,
            key: helper.key,
        })
    }
}

impl EvmStorageKey {
    pub fn new(
        chain_id: ChainId,
        block_number: BlockNumber,
        address: Address,
        key: StorageKey,
    ) -> Self {
        Self {
            chain_id,
            block_number,
            address,
            key,
        }
    }

    pub fn hash_key(&self) -> B256 {
        let mut keccak = Keccak256::new();
        keccak.update([BlockSampledCollectionType::Storage.to_u8()]);
        keccak.update(self.chain_id.to_be_bytes());
        keccak.update(self.block_number.to_be_bytes());
        keccak.update(self.address);
        keccak.update(self.key);
        keccak.finalize()
    }
}

/// Key for fetching transaction from provider.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct EvmBlockTxKey {
    pub chain_id: ChainId,
    pub block_number: BlockNumber,
    pub index: u64,
}

impl<'de> Deserialize<'de> for EvmBlockTxKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            chain_id: String,
            block_number: BlockNumber,
            index: u64,
        }

        let helper = Helper::deserialize(deserializer)?;

        Ok(EvmBlockTxKey {
            chain_id: ChainId::from_hex_str(&helper.chain_id).expect("invalid deserialize"),
            block_number: helper.block_number,
            index: helper.index,
        })
    }
}

impl EvmBlockTxKey {
    pub fn new(chain_id: ChainId, block_number: BlockNumber, index: u64) -> Self {
        Self {
            chain_id,
            block_number,
            index,
        }
    }

    pub fn hash_key(&self) -> B256 {
        let mut keccak = Keccak256::new();
        keccak.update([TransactionsCollectionType::Transactions.to_u8()]);
        keccak.update(self.chain_id.to_be_bytes());
        keccak.update(self.block_number.to_be_bytes());
        keccak.update(self.index.to_be_bytes());
        keccak.finalize()
    }
}

/// Key for fetching transaction receipt from provider.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct EvmBlockReceiptKey {
    pub chain_id: ChainId,
    pub block_number: BlockNumber,
    pub index: u64,
}

impl<'de> Deserialize<'de> for EvmBlockReceiptKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            chain_id: String,
            block_number: BlockNumber,
            index: u64,
        }

        let helper = Helper::deserialize(deserializer)?;

        Ok(EvmBlockReceiptKey {
            chain_id: ChainId::from_hex_str(&helper.chain_id).expect("invalid deserialize"),
            block_number: helper.block_number,
            index: helper.index,
        })
    }
}

impl EvmBlockReceiptKey {
    pub fn new(chain_id: ChainId, block_number: BlockNumber, index: u64) -> Self {
        Self {
            chain_id,
            block_number,
            index,
        }
    }

    pub fn hash_key(&self) -> B256 {
        let mut keccak = Keccak256::new();
        keccak.update([TransactionsCollectionType::TransactionReceipts.to_u8()]);
        keccak.update(self.chain_id.to_be_bytes());
        keccak.update(self.block_number.to_be_bytes());
        keccak.update(self.index.to_be_bytes());
        keccak.finalize()
    }
}

#[derive(Hash, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(tag = "type", content = "key")]
pub enum FetchKeyEnvelope {
    #[serde(rename = "EvmHeaderKey")]
    EvmHeader(EvmHeaderKey),
    #[serde(rename = "EvmAccountKey")]
    EvmAccount(EvmAccountKey),
    #[serde(rename = "EvmStorageKey")]
    EvmStorage(EvmStorageKey),
    #[serde(rename = "EvmBlockTxKey")]
    EvmTx(EvmBlockTxKey),
    #[serde(rename = "EvmBlockReceiptKey")]
    EvmTxReceipt(EvmBlockReceiptKey),
    #[serde(rename = "StarknetHeaderKey")]
    StarknetHeader(StarknetHeaderKey),
    #[serde(rename = "StarknetStorageKey")]
    StarknetStorage(StarknetStorageKey),
}

impl FromStr for FetchKeyEnvelope {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('_').collect();
        if parts.len() < 2 {
            anyhow::bail!("Invalid fetch key envelope: {}", s);
        }

        let chain_id = parts[0].parse()?;
        let block_number = parts[1].parse()?;

        match parts.len() {
            2 => Ok(FetchKeyEnvelope::EvmHeader(EvmHeaderKey {
                chain_id,
                block_number,
            })),
            3 => {
                let address = parts[2].parse()?;
                Ok(FetchKeyEnvelope::EvmAccount(EvmAccountKey {
                    chain_id,
                    block_number,
                    address,
                }))
            }
            4 => {
                let address = parts[2].parse()?;
                let key = parts[3].parse()?;
                Ok(FetchKeyEnvelope::EvmStorage(EvmStorageKey {
                    chain_id,
                    block_number,
                    address,
                    key,
                }))
            }
            _ => anyhow::bail!("Invalid fetch key envelope: {}", s),
        }
    }
}

impl FetchKeyEnvelope {
    /// Get the chain id from the fetch key.
    pub fn get_chain_id(&self) -> ChainId {
        match self {
            FetchKeyEnvelope::EvmHeader(key) => key.chain_id,
            FetchKeyEnvelope::EvmAccount(key) => key.chain_id,
            FetchKeyEnvelope::EvmStorage(key) => key.chain_id,
            FetchKeyEnvelope::EvmTx(key) => key.chain_id,
            FetchKeyEnvelope::EvmTxReceipt(key) => key.chain_id,
            FetchKeyEnvelope::StarknetHeader(key) => key.chain_id,
            FetchKeyEnvelope::StarknetStorage(key) => key.chain_id,
        }
    }
}

#[cfg(test)]
mod tests {

    use alloy::primitives::b256;

    use super::*;

    #[test]
    fn test_hash_header_key() {
        let header_key = EvmHeaderKey::new(ChainId::EthereumMainnet, 100);
        let header_key_hash = header_key.hash_key();
        assert_eq!(
            header_key_hash,
            b256!("6bc10761f4d566044340a77d51c936b8d1ee7d4ebfb3e62873d7d37eb8964505")
        )
    }

    #[test]
    fn test_hash_account_key() {
        let account_key = EvmAccountKey::new(ChainId::EthereumMainnet, 100, Address::ZERO);
        let account_key_hash = account_key.hash_key();
        assert_eq!(
            account_key_hash,
            b256!("044229e95af51ab44d057270d10f948d7e6f0b98075abb702d535e237b573794")
        )
    }

    #[test]
    fn test_hash_storage_key() {
        let storage_key =
            EvmStorageKey::new(ChainId::EthereumMainnet, 100, Address::ZERO, B256::ZERO);
        let storage_key_hash = storage_key.hash_key();
        assert_eq!(
            storage_key_hash,
            b256!("c0aca94acc508394ff0ce22ebf1bbe1db21c35e6e4fa70d72bd6cac0742381b3")
        )
    }

    #[test]
    fn test_hash_tx_key() {
        let tx_key = EvmBlockTxKey::new(ChainId::EthereumMainnet, 100, 1);
        let tx_key_hash = tx_key.hash_key();
        assert_eq!(
            tx_key_hash,
            b256!("103cc4b0d6d6e45d7189c99425aa02b7ebd9b861e9bb4331a02364980e02481f")
        )
    }

    #[test]
    fn test_hash_tx_receipt_key() {
        let tx_receipt = EvmBlockReceiptKey::new(ChainId::EthereumMainnet, 100, 1);
        let tx_receipt_hash = tx_receipt.hash_key();
        assert_eq!(
            tx_receipt_hash,
            b256!("9b6f454da0ab2264a9a6897abd039c322de33972442eac072635c66ca5c7db86")
        )
    }

    #[test]
    fn test_parse_json_header_key() {
        let json =
            r#"{"type": "EvmHeaderKey", "key": {"chain_id": "0xAA36A7", "block_number": 100}}"#;
        let parsed: FetchKeyEnvelope = serde_json::from_str(json).unwrap();
        assert_eq!(
            parsed,
            FetchKeyEnvelope::EvmHeader(EvmHeaderKey::new(ChainId::EthereumSepolia, 100))
        );
    }

    #[test]
    fn test_parse_json_account_key() {
        let json = r#"{"type": "EvmAccountKey", "key": {"chain_id": "0x1", "block_number": 100, "address": "0x0000000000000000000000000000000000000000"}}"#;
        let parsed: FetchKeyEnvelope = serde_json::from_str(json).unwrap();
        assert_eq!(
            parsed,
            FetchKeyEnvelope::EvmAccount(EvmAccountKey::new(
                ChainId::EthereumMainnet,
                100,
                Address::ZERO
            ))
        );
    }

    #[test]
    fn test_parse_json_storage_key() {
        let json = r#"{"type": "EvmStorageKey", "key": {"chain_id": "0x1", "block_number": 100, "address": "0x0000000000000000000000000000000000000000", "key": "0x0000000000000000000000000000000000000000000000000000000000000000"}}"#;
        let parsed: FetchKeyEnvelope = serde_json::from_str(json).unwrap();
        assert_eq!(
            parsed,
            FetchKeyEnvelope::EvmStorage(EvmStorageKey::new(
                ChainId::EthereumMainnet,
                100,
                Address::ZERO,
                B256::ZERO
            ))
        );
    }

    #[test]
    fn test_parse_json_tx_key() {
        let json = r#"{"type": "EvmBlockTxKey", "key": {"chain_id": "0x1", "block_number": 100, "index": 1}}"#;
        let parsed: FetchKeyEnvelope = serde_json::from_str(json).unwrap();
        assert_eq!(
            parsed,
            FetchKeyEnvelope::EvmTx(EvmBlockTxKey::new(ChainId::EthereumMainnet, 100, 1))
        );
    }

    #[test]
    fn test_parse_json_tx_receipt_key() {
        let json = r#"{"type": "EvmBlockReceiptKey", "key": {"chain_id": "0x1", "block_number": 100, "index": 1}}"#;
        let parsed: FetchKeyEnvelope = serde_json::from_str(json).unwrap();
        assert_eq!(
            parsed,
            FetchKeyEnvelope::EvmTxReceipt(EvmBlockReceiptKey::new(
                ChainId::EthereumMainnet,
                100,
                1
            ))
        );
    }

    #[test]
    fn test_parse_json_sn_header_key() {
        let json = r#"{"type": "StarknetHeaderKey", "key": {"chain_id": "0x534E5F5345504F4C4941", "block_number": 4660 }}"#;
        let parsed: FetchKeyEnvelope = serde_json::from_str(json).unwrap();
        assert_eq!(
            parsed,
            FetchKeyEnvelope::StarknetHeader(StarknetHeaderKey::new(
                ChainId::StarknetSepolia,
                4660
            ))
        );
    }

    #[test]
    fn test_parse_json_sn_storage_key() {
        let json = r#"{"type": "StarknetStorageKey", "key": {"chain_id": "0x534E5F5345504F4C4941", "block_number": 4660, "address": "0x1234", "key": "0x1234" }}"#;
        let parsed: FetchKeyEnvelope = serde_json::from_str(json).unwrap();
        assert_eq!(
            parsed,
            FetchKeyEnvelope::StarknetStorage(StarknetStorageKey::new(
                ChainId::StarknetSepolia,
                4660,
                Felt::from_hex("0x1234").unwrap(),
                Felt::from_hex("0x1234").unwrap()
            ))
        );
    }
}
