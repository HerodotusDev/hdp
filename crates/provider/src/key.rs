//! Provider keys for fetching data from memoizer and rpc.
//! Only used for context of Module Compiler
//!
//! TODO: need to sync with how bootloader will emit the keys

use std::{hash::Hash, str::FromStr};

use alloy::primitives::{Address, BlockNumber, ChainId, Keccak256, StorageKey, B256};
use serde::{Deserialize, Serialize};

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
            b256!("06244d81b463cd9e199e3d3845d948bf87e094b4cd407c87238b52e4ec017e06")
        )
    }

    #[test]
    fn test_hash_account_key() {
        let account_key = AccountMemorizerKey::new(1, 100, Address::ZERO);
        let account_key_hash = account_key.hash_key();
        assert_eq!(
            account_key_hash,
            b256!("dbea9b2e992075e52528a88d0e4ed0471599bc7a6b790a947a361c88051d5ae0")
        )
    }
}
