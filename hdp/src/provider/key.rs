//! Provider keys for fetching data from memoizer and rpc.
//! Only used for context of Module Compiler
//!
//! TODO: need to sync with how bootloader will emit the keys

use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    str::FromStr,
};

use crate::primitives::task::datalake::{
    block_sampled::BlockSampledCollectionType, transactions::TransactionsCollectionType,
};
use alloy::primitives::{Address, BlockNumber, ChainId, Keccak256, StorageKey, B256};
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
    let mut chain_id_map: HashMap<u64, CategorizedFetchKeys> = std::collections::HashMap::new();

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct HeaderMemorizerKey {
    pub chain_id: ChainId,
    pub block_number: BlockNumber,
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AccountMemorizerKey {
    pub chain_id: ChainId,
    pub block_number: BlockNumber,
    pub address: Address,
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct StorageMemorizerKey {
    pub chain_id: ChainId,
    pub block_number: BlockNumber,
    pub address: Address,
    pub key: StorageKey,
}

impl StorageMemorizerKey {
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TxMemorizerKey {
    pub chain_id: ChainId,
    pub block_number: BlockNumber,
    pub tx_index: u64,
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TxReceiptMemorizerKey {
    pub chain_id: ChainId,
    pub block_number: BlockNumber,
    pub tx_index: u64,
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
                let key = parts[3].parse()?;
                Ok(FetchKeyEnvelope::Storage(StorageMemorizerKey {
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
        let header_key = HeaderMemorizerKey::new(1, 100);
        let header_key_hash = header_key.hash_key();
        assert_eq!(
            header_key_hash,
            b256!("205342692a0b320915750abf5ad47709ea4c6c10e18802d5981070e47bd6f0c1")
        )
    }

    #[test]
    fn test_hash_account_key() {
        let account_key = AccountMemorizerKey::new(1, 100, Address::ZERO);
        let account_key_hash = account_key.hash_key();
        assert_eq!(
            account_key_hash,
            b256!("4298fef03738f1a6d9f77e3d1ad8cf7906a60f4f9ecd63b34cb2a1ac61353e7a")
        )
    }

    #[test]
    fn test_hash_storage_key() {
        let storage_key = StorageMemorizerKey::new(1, 100, Address::ZERO, B256::ZERO);
        let storage_key_hash = storage_key.hash_key();
        assert_eq!(
            storage_key_hash,
            b256!("22133ae0a8c6964d0e4fc28659442e311c4a9f6a82f16ea9f103e36a1085c11f")
        )
    }

    #[test]
    fn test_hash_tx_key() {
        let tx_key = TxMemorizerKey::new(1, 100, 1);
        let tx_key_hash = tx_key.hash_key();
        assert_eq!(
            tx_key_hash,
            b256!("487ea7bf96eb1280f1075498855b55ec61ba7d354b5260e2504ef51140e0df63")
        )
    }

    #[test]
    fn test_hash_tx_receipt_key() {
        let tx_receipt = TxReceiptMemorizerKey::new(1, 100, 1);
        let tx_receipt_hash = tx_receipt.hash_key();
        assert_eq!(
            tx_receipt_hash,
            b256!("beac44bc7092d6c5c2c9ce2d6957c9b962ad4654a65c9b9a9b19a9c278ee5a83")
        )
    }

    #[test]
    fn test_fetch_key_envelop() {}
}
