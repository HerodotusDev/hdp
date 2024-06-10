//! Provider keys for fetching data from memoizer and rpc.

use std::str::FromStr;

use alloy_primitives::{Address, BlockNumber, ChainId, StorageKey};

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

impl_hash_for_provider_key!(HeaderProviderKey {
    chain_id,
    block_number
});

impl_hash_for_provider_key!(AccountProviderKey {
    chain_id,
    block_number,
    address
});

impl_hash_for_provider_key!(StorageProviderKey {
    chain_id,
    block_number,
    address,
    key
});

/// Key for fetching block header from provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderProviderKey {
    pub chain_id: ChainId,
    pub block_number: BlockNumber,
}

// TODO: Temporary implemented from string approach, but need to sync with how bootloader will emit the keys.
impl FromStr for HeaderProviderKey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('_').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid header provider key: {}", s);
        }

        let chain_id = parts[0].parse()?;
        let block_number = parts[1].parse()?;

        Ok(HeaderProviderKey {
            chain_id,
            block_number,
        })
    }
}

/// Key for fetching account from provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountProviderKey {
    pub chain_id: ChainId,
    pub block_number: BlockNumber,
    pub address: Address,
}

impl FromStr for AccountProviderKey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('_').collect();
        if parts.len() != 3 {
            anyhow::bail!("Invalid account provider key: {}", s);
        }

        let chain_id = parts[0].parse()?;
        let block_number = parts[1].parse()?;
        let address = parts[2].parse()?;

        Ok(AccountProviderKey {
            chain_id,
            block_number,
            address,
        })
    }
}

/// Key for fetching storage value from provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageProviderKey {
    pub chain_id: ChainId,
    pub block_number: BlockNumber,
    pub address: Address,
    pub key: StorageKey,
}

impl FromStr for StorageProviderKey {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('_').collect();
        if parts.len() != 4 {
            anyhow::bail!("Invalid storage provider key: {}", s);
        }

        let chain_id = parts[0].parse()?;
        let block_number = parts[1].parse()?;
        let address = parts[2].parse()?;
        let key = parts[3].parse()?;

        Ok(StorageProviderKey {
            chain_id,
            block_number,
            address,
            key,
        })
    }
}

pub trait FetchKey: std::fmt::Debug + std::hash::Hash + Clone + FromStr {}

impl FetchKey for HeaderProviderKey {}
impl FetchKey for AccountProviderKey {}
impl FetchKey for StorageProviderKey {}
