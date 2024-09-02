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

/// This is keys that are categorized into different subsets of keys.
#[derive(Debug, Default)]
pub struct CategorizedFetchKeys {
    pub headers: HashSet<HeaderMemorizerKey>,
    pub accounts: HashSet<AccountMemorizerKey>,
    pub storage: HashSet<StorageMemorizerKey>,
    pub txs: HashSet<TxMemorizerKey>,
    pub tx_receipts: HashSet<TxReceiptMemorizerKey>,
}

impl CategorizedFetchKeys {
    pub fn new(
        headers: HashSet<HeaderMemorizerKey>,
        accounts: HashSet<AccountMemorizerKey>,
        storage: HashSet<StorageMemorizerKey>,
        txs: HashSet<TxMemorizerKey>,
        tx_receipts: HashSet<TxReceiptMemorizerKey>,
    ) -> Self {
        Self {
            headers,
            accounts,
            storage,
            txs,
            tx_receipts,
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
            FetchKeyEnvelope::Header(header_key) => {
                target_categorized_fetch_keys.headers.insert(header_key);
            }
            FetchKeyEnvelope::Account(account_key) => {
                target_categorized_fetch_keys
                    .headers
                    .insert(HeaderMemorizerKey::new(
                        account_key.chain_id,
                        account_key.block_number,
                    ));
                target_categorized_fetch_keys.accounts.insert(account_key);
            }
            FetchKeyEnvelope::Storage(storage_key) => {
                target_categorized_fetch_keys
                    .headers
                    .insert(HeaderMemorizerKey::new(
                        storage_key.chain_id,
                        storage_key.block_number,
                    ));
                target_categorized_fetch_keys.storage.insert(storage_key);
            }
            FetchKeyEnvelope::Tx(tx_key) => {
                target_categorized_fetch_keys
                    .headers
                    .insert(HeaderMemorizerKey::new(
                        tx_key.chain_id,
                        tx_key.block_number,
                    ));
                target_categorized_fetch_keys.txs.insert(tx_key);
            }
            FetchKeyEnvelope::TxReceipt(tx_receipt_key) => {
                target_categorized_fetch_keys
                    .headers
                    .insert(HeaderMemorizerKey::new(
                        tx_receipt_key.chain_id,
                        tx_receipt_key.block_number,
                    ));
                target_categorized_fetch_keys
                    .tx_receipts
                    .insert(tx_receipt_key);
            }
        }
    }
    chain_id_map.into_iter().collect()
}

/// Key for fetching block header from provider.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct HeaderMemorizerKey {
    pub chain_id: ChainId,
    pub block_number: BlockNumber,
}

impl<'de> Deserialize<'de> for HeaderMemorizerKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            chain_id: u128,
            block_number: BlockNumber,
        }

        let helper = Helper::deserialize(deserializer)?;

        Ok(HeaderMemorizerKey {
            chain_id: ChainId::from_numeric_id(helper.chain_id).expect("invalid deserialize"),
            block_number: helper.block_number,
        })
    }
}

impl HeaderMemorizerKey {
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
pub struct AccountMemorizerKey {
    pub chain_id: ChainId,
    pub block_number: BlockNumber,
    pub address: Address,
}

impl<'de> Deserialize<'de> for AccountMemorizerKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            chain_id: u128,
            block_number: BlockNumber,
            address: Address,
        }

        let helper = Helper::deserialize(deserializer)?;

        Ok(AccountMemorizerKey {
            chain_id: ChainId::from_numeric_id(helper.chain_id).expect("invalid deserialize"),
            block_number: helper.block_number,
            address: helper.address,
        })
    }
}

impl AccountMemorizerKey {
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
pub struct StorageMemorizerKey {
    pub chain_id: ChainId,
    pub block_number: BlockNumber,
    pub address: Address,
    pub storage_key: StorageKey,
}

impl<'de> Deserialize<'de> for StorageMemorizerKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            chain_id: u128,
            block_number: BlockNumber,
            address: Address,
            storage_key: StorageKey,
        }

        let helper = Helper::deserialize(deserializer)?;

        Ok(StorageMemorizerKey {
            chain_id: ChainId::from_numeric_id(helper.chain_id).expect("invalid deserialize"),
            block_number: helper.block_number,
            address: helper.address,
            storage_key: helper.storage_key,
        })
    }
}

impl StorageMemorizerKey {
    pub fn new(
        chain_id: ChainId,
        block_number: BlockNumber,
        address: Address,
        storage_key: StorageKey,
    ) -> Self {
        Self {
            chain_id,
            block_number,
            address,
            storage_key,
        }
    }

    pub fn hash_key(&self) -> B256 {
        let mut keccak = Keccak256::new();
        keccak.update([BlockSampledCollectionType::Storage.to_u8()]);
        keccak.update(self.chain_id.to_be_bytes());
        keccak.update(self.block_number.to_be_bytes());
        keccak.update(self.address);
        keccak.update(self.storage_key);
        keccak.finalize()
    }
}

/// Key for fetching transaction from provider.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct TxMemorizerKey {
    pub chain_id: ChainId,
    pub block_number: BlockNumber,
    pub tx_index: u64,
}

impl<'de> Deserialize<'de> for TxMemorizerKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            chain_id: u128,
            block_number: BlockNumber,
            tx_index: u64,
        }

        let helper = Helper::deserialize(deserializer)?;

        Ok(TxMemorizerKey {
            chain_id: ChainId::from_numeric_id(helper.chain_id).expect("invalid deserialize"),
            block_number: helper.block_number,
            tx_index: helper.tx_index,
        })
    }
}

impl TxMemorizerKey {
    pub fn new(chain_id: ChainId, block_number: BlockNumber, tx_index: u64) -> Self {
        Self {
            chain_id,
            block_number,
            tx_index,
        }
    }

    pub fn hash_key(&self) -> B256 {
        let mut keccak = Keccak256::new();
        keccak.update([TransactionsCollectionType::Transactions.to_u8()]);
        keccak.update(self.chain_id.to_be_bytes());
        keccak.update(self.block_number.to_be_bytes());
        keccak.update(self.tx_index.to_be_bytes());
        keccak.finalize()
    }
}

/// Key for fetching transaction receipt from provider.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash)]
pub struct TxReceiptMemorizerKey {
    pub chain_id: ChainId,
    pub block_number: BlockNumber,
    pub tx_index: u64,
}

impl<'de> Deserialize<'de> for TxReceiptMemorizerKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            chain_id: u128,
            block_number: BlockNumber,
            tx_index: u64,
        }

        let helper = Helper::deserialize(deserializer)?;

        Ok(TxReceiptMemorizerKey {
            chain_id: ChainId::from_numeric_id(helper.chain_id).expect("invalid deserialize"),
            block_number: helper.block_number,
            tx_index: helper.tx_index,
        })
    }
}

impl TxReceiptMemorizerKey {
    pub fn new(chain_id: ChainId, block_number: BlockNumber, tx_index: u64) -> Self {
        Self {
            chain_id,
            block_number,
            tx_index,
        }
    }

    pub fn hash_key(&self) -> B256 {
        let mut keccak = Keccak256::new();
        keccak.update([TransactionsCollectionType::TransactionReceipts.to_u8()]);
        keccak.update(self.chain_id.to_be_bytes());
        keccak.update(self.block_number.to_be_bytes());
        keccak.update(self.tx_index.to_be_bytes());
        keccak.finalize()
    }
}

#[derive(Hash, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(tag = "type", content = "key")]
pub enum FetchKeyEnvelope {
    #[serde(rename = "HeaderMemorizerKey")]
    Header(HeaderMemorizerKey),
    #[serde(rename = "AccountMemorizerKey")]
    Account(AccountMemorizerKey),
    #[serde(rename = "StorageMemorizerKey")]
    Storage(StorageMemorizerKey),
    #[serde(rename = "TxMemorizerKey")]
    Tx(TxMemorizerKey),
    #[serde(rename = "TxReceiptMemorizerKey")]
    TxReceipt(TxReceiptMemorizerKey),
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
            2 => Ok(FetchKeyEnvelope::Header(HeaderMemorizerKey {
                chain_id,
                block_number,
            })),
            3 => {
                let address = parts[2].parse()?;
                Ok(FetchKeyEnvelope::Account(AccountMemorizerKey {
                    chain_id,
                    block_number,
                    address,
                }))
            }
            4 => {
                let address = parts[2].parse()?;
                let storage_key = parts[3].parse()?;
                Ok(FetchKeyEnvelope::Storage(StorageMemorizerKey {
                    chain_id,
                    block_number,
                    address,
                    storage_key,
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
            FetchKeyEnvelope::Header(key) => key.chain_id,
            FetchKeyEnvelope::Account(key) => key.chain_id,
            FetchKeyEnvelope::Storage(key) => key.chain_id,
            FetchKeyEnvelope::Tx(key) => key.chain_id,
            FetchKeyEnvelope::TxReceipt(key) => key.chain_id,
        }
    }
}

#[cfg(test)]
mod tests {

    use alloy::primitives::b256;

    use super::*;

    #[test]
    fn test_hash_header_key() {
        let header_key = HeaderMemorizerKey::new(ChainId::EthereumMainnet, 100);
        let header_key_hash = header_key.hash_key();
        assert_eq!(
            header_key_hash,
            b256!("6bc10761f4d566044340a77d51c936b8d1ee7d4ebfb3e62873d7d37eb8964505")
        )
    }

    #[test]
    fn test_hash_account_key() {
        let account_key = AccountMemorizerKey::new(ChainId::EthereumMainnet, 100, Address::ZERO);
        let account_key_hash = account_key.hash_key();
        assert_eq!(
            account_key_hash,
            b256!("044229e95af51ab44d057270d10f948d7e6f0b98075abb702d535e237b573794")
        )
    }

    #[test]
    fn test_hash_storage_key() {
        let storage_key =
            StorageMemorizerKey::new(ChainId::EthereumMainnet, 100, Address::ZERO, B256::ZERO);
        let storage_key_hash = storage_key.hash_key();
        assert_eq!(
            storage_key_hash,
            b256!("c0aca94acc508394ff0ce22ebf1bbe1db21c35e6e4fa70d72bd6cac0742381b3")
        )
    }

    #[test]
    fn test_hash_tx_key() {
        let tx_key = TxMemorizerKey::new(ChainId::EthereumMainnet, 100, 1);
        let tx_key_hash = tx_key.hash_key();
        assert_eq!(
            tx_key_hash,
            b256!("103cc4b0d6d6e45d7189c99425aa02b7ebd9b861e9bb4331a02364980e02481f")
        )
    }

    #[test]
    fn test_hash_tx_receipt_key() {
        let tx_receipt = TxReceiptMemorizerKey::new(ChainId::EthereumMainnet, 100, 1);
        let tx_receipt_hash = tx_receipt.hash_key();
        assert_eq!(
            tx_receipt_hash,
            b256!("9b6f454da0ab2264a9a6897abd039c322de33972442eac072635c66ca5c7db86")
        )
    }

    #[test]
    fn test_parse_json_header_key() {
        let json =
            r#"{"type": "HeaderMemorizerKey", "key": {"chain_id": 11155111, "block_number": 100}}"#;
        let parsed: FetchKeyEnvelope = serde_json::from_str(json).unwrap();
        assert_eq!(
            parsed,
            FetchKeyEnvelope::Header(HeaderMemorizerKey::new(ChainId::EthereumSepolia, 100))
        );
    }

    #[test]
    fn test_parse_json_account_key() {
        let json = r#"{"type": "AccountMemorizerKey", "key": {"chain_id": 1, "block_number": 100, "address": "0x0000000000000000000000000000000000000000"}}"#;
        let parsed: FetchKeyEnvelope = serde_json::from_str(json).unwrap();
        assert_eq!(
            parsed,
            FetchKeyEnvelope::Account(AccountMemorizerKey::new(
                ChainId::EthereumMainnet,
                100,
                Address::ZERO
            ))
        );
    }

    #[test]
    fn test_parse_json_storage_key() {
        let json = r#"{"type": "StorageMemorizerKey", "key": {"chain_id": 1, "block_number": 100, "address": "0x0000000000000000000000000000000000000000", "storage_key": "0x0000000000000000000000000000000000000000000000000000000000000000"}}"#;
        let parsed: FetchKeyEnvelope = serde_json::from_str(json).unwrap();
        assert_eq!(
            parsed,
            FetchKeyEnvelope::Storage(StorageMemorizerKey::new(
                ChainId::EthereumMainnet,
                100,
                Address::ZERO,
                B256::ZERO
            ))
        );
    }

    #[test]
    fn test_parse_json_tx_key() {
        let json = r#"{"type": "TxMemorizerKey", "key": {"chain_id": 1, "block_number": 100, "tx_index": 1}}"#;
        let parsed: FetchKeyEnvelope = serde_json::from_str(json).unwrap();
        assert_eq!(
            parsed,
            FetchKeyEnvelope::Tx(TxMemorizerKey::new(ChainId::EthereumMainnet, 100, 1))
        );
    }

    #[test]
    fn test_parse_json_tx_receipt_key() {
        let json = r#"{"type": "TxReceiptMemorizerKey", "key": {"chain_id": 1, "block_number": 100, "tx_index": 1}}"#;
        let parsed: FetchKeyEnvelope = serde_json::from_str(json).unwrap();
        assert_eq!(
            parsed,
            FetchKeyEnvelope::TxReceipt(TxReceiptMemorizerKey::new(
                ChainId::EthereumMainnet,
                100,
                1
            ))
        );
    }
}
