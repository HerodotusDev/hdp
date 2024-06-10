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

/// Key for fetching account from provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountProviderKey {
    pub chain_id: ChainId,
    pub block_number: BlockNumber,
    pub address: Address,
}

/// Key for fetching storage value from provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageProviderKey {
    pub chain_id: ChainId,
    pub block_number: BlockNumber,
    pub address: Address,
    pub key: StorageKey,
}

// TODO: Temporary implemented from string approach, but need to sync with how bootloader will emit the keys
pub enum FetchKeyEnvelope {
    Header(HeaderProviderKey),
    Account(AccountProviderKey),
    Storage(StorageProviderKey),
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
            2 => Ok(FetchKeyEnvelope::Header(HeaderProviderKey {
                chain_id,
                block_number,
            })),
            3 => {
                let address = parts[2].parse()?;
                Ok(FetchKeyEnvelope::Account(AccountProviderKey {
                    chain_id,
                    block_number,
                    address,
                }))
            }
            4 => {
                let address = parts[2].parse()?;
                let key = parts[3].parse()?;
                Ok(FetchKeyEnvelope::Storage(StorageProviderKey {
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
