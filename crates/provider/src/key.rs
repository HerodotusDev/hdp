//! Provider keys for fetching data from memoizer and rpc.
//! Only used for context of Module Compiler
//!
//! TODO: need to sync with how bootloader will emit the keys

use std::str::FromStr;

use alloy::primitives::{Address, BlockNumber, ChainId, StorageKey};
use serde::{Deserialize, Serialize};

macro_rules! impl_hash_for_provider_key {
    // Match a struct with an identifier and any number of fields.
    ($key:ident { $( $field:ident ),* }) => {
        impl std::hash::Hash for $key {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                $( self.$field.hash(state); )*
            }
        }
    };
}

impl_hash_for_provider_key!(HeaderMemorizerKey {
    chain_id,
    block_number
});

impl_hash_for_provider_key!(AccountMemorizerKey {
    chain_id,
    block_number,
    address
});

impl_hash_for_provider_key!(StorageMemorizerKey {
    chain_id,
    block_number,
    address,
    key
});

impl_hash_for_provider_key!(TxMemorizerKey {
    chain_id,
    block_number,
    tx_index
});

impl_hash_for_provider_key!(TxReceiptMemorizerKey {
    chain_id,
    block_number,
    tx_index
});

/// Key for fetching block header from provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
}

/// Key for fetching account from provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
}

/// Key for fetching storage value from provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
}

/// Key for fetching transaction from provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
}

/// Key for fetching transaction receipt from provider.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
}

#[derive(Hash, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(tag = "type", content = "key")]
pub enum FetchKeyEnvelope {
    Header(HeaderMemorizerKey),
    Account(AccountMemorizerKey),
    Storage(StorageMemorizerKey),
    Tx(TxMemorizerKey),
    TxReceipt(TxReceiptMemorizerKey),
}

// TODO: Temporary implemented from string approach, but need to sync with how bootloader will emit the keys
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
