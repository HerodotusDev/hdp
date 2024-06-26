//! RPC provider for fetching data from Ethereum RPC
//! It is a wrapper around the alloy provider, using eth_getProof for fetching account and storage proofs
//!
//! How to use:
//! ```rust
//! use reqwest::Url;
//! use hdp_provider::evm::rpc::RpcProvider;
//! use alloy::primitives::Address;
//!
//! async fn call_provider(url: Url, chunk_size: u64, block_range_start: u64, block_range_end: u64, increment: u64, address: Address) {
//!         let provider = RpcProvider::new(url, chunk_size);
//!         let target_block_range = (block_range_start..=block_range_end).collect::<Vec<u64>>();
//!         let result = provider.get_account_proofs(target_block_range, address).await;
//!         match result {
//!             Ok(proofs) => println!("Fetched proofs: {:?}", proofs),
//!             Err(e) => eprintln!("Error fetching proofs: {:?}", e),
//!         }
//! }
//! ```

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
};

use alloy::{
    primitives::{Address, BlockNumber, StorageKey},
    providers::{Provider, RootProvider},
    rpc::types::EIP1186AccountProofResponse,
    transports::{
        http::{Client, Http},
        RpcError, TransportErrorKind,
    },
};
use futures::future::join_all;
use reqwest::Url;
use tokio::sync::{
    mpsc::{self, Sender},
    RwLock,
};
use tracing::{debug, info};

use crate::errors::ProviderError;

#[derive(Clone)]
pub struct RpcProvider {
    provider: RootProvider<Http<Client>>,
    chunk_size: u64,
}

impl RpcProvider {
    pub fn new(rpc_url: Url, chunk_size: u64) -> Self {
        let provider = RootProvider::new_http(rpc_url);
        Self {
            provider,
            chunk_size,
        }
    }

    /// Get account with proof in given vector of blocks
    pub async fn get_account_proofs(
        &self,
        blocks: Vec<u64>,
        address: Address,
    ) -> Result<HashMap<u64, EIP1186AccountProofResponse>, ProviderError> {
        let start_fetch = Instant::now();

        let (rpc_sender, mut rx) = mpsc::channel::<(BlockNumber, EIP1186AccountProofResponse)>(32);
        self._get_account_proofs(rpc_sender, blocks, address);
        let mut result = HashMap::new();

        while let Some((block_number, proof)) = rx.recv().await {
            result.insert(block_number, proof);
        }
        let duration = start_fetch.elapsed();
        info!("Time taken (Account Fetch): {:?}", duration);

        Ok(result)
    }

    /// Get storage with proof in given vector of blocks and slot
    pub async fn get_storage_proofs(
        &self,
        block_range: Vec<u64>,
        address: Address,
        storage_key: StorageKey,
    ) -> Result<HashMap<u64, EIP1186AccountProofResponse>, ProviderError> {
        let start_fetch = Instant::now();

        let (rpc_sender, mut rx) = mpsc::channel::<(BlockNumber, EIP1186AccountProofResponse)>(32);
        self._get_storage_proofs(rpc_sender, block_range, address, storage_key);

        let mut result = HashMap::new();

        while let Some((block_number, proof)) = rx.recv().await {
            result.insert(block_number, proof);
        }
        let duration = start_fetch.elapsed();
        info!("Time taken (Storage Fetch): {:?}", duration);

        Ok(result)
    }

    fn _get_account_proofs(
        &self,
        rpc_sender: Sender<(BlockNumber, EIP1186AccountProofResponse)>,
        blocks: Vec<u64>,
        address: Address,
    ) {
        let chunk_size = self.chunk_size;
        let provider_clone = self.provider.clone(); // Clone provider here

        debug!(
            "Fetching account proofs for {} chunk size: {}",
            address, chunk_size
        );

        tokio::spawn(async move {
            let mut try_count = 0;
            let blocks_map = Arc::new(RwLock::new(HashSet::<u64>::new()));

            while blocks_map.read().await.len() < blocks.len() {
                try_count += 1;
                if try_count > 50 {
                    panic!("❗️❗️❗️ Too many retries, failed to fetch all blocks")
                }
                let fetched_blocks_clone = blocks_map.read().await.clone();
                let blocks_to_fetch: Vec<u64> = blocks
                    .iter()
                    .filter(|block_number| !fetched_blocks_clone.contains(*block_number))
                    .take(chunk_size as usize)
                    .cloned()
                    .collect();

                let fetch_futures = blocks_to_fetch
                    .into_iter()
                    .map(|block_number| {
                        let fetched_blocks_clone = blocks_map.clone();
                        let rpc_sender = rpc_sender.clone();
                        let provider_clone = provider_clone.clone(); // Use cloned provider
                        async move {
                            let account_from_rpc = provider_clone
                                .get_proof(address, vec![])
                                .block_id(block_number.into())
                                .await;
                            match account_from_rpc {
                                Ok(account_from_rpc) => {
                                    let mut blocks_identifier = fetched_blocks_clone.write().await;
                                    rpc_sender
                                        .send((block_number, account_from_rpc))
                                        .await
                                        .unwrap();
                                    blocks_identifier.insert(block_number);
                                }
                                Err(e) => {
                                    if let Some(backoff) = handle_error(e) {
                                        let mut delay = backoff;
                                        while delay <= 4 {
                                            tokio::time::sleep(Duration::from_nanos(delay)).await;
                                            delay *= 2;
                                        }
                                    }
                                }
                            }
                        }
                    })
                    .collect::<Vec<_>>();

                join_all(fetch_futures).await;
            }
        });
    }

    fn _get_storage_proofs(
        &self,
        rpc_sender: Sender<(BlockNumber, EIP1186AccountProofResponse)>,
        blocks: Vec<u64>,
        address: Address,
        storage_key: StorageKey,
    ) {
        let chunk_size = self.chunk_size;
        let provider_clone = self.provider.clone(); // Clone provider here

        debug!(
            "Fetching storage proofs for {} chunk size: {}",
            address, chunk_size
        );

        tokio::spawn(async move {
            let mut try_count = 0;
            let blocks_map = Arc::new(RwLock::new(HashSet::<u64>::new()));

            while blocks_map.read().await.len() < blocks.len() {
                try_count += 1;
                if try_count > 50 {
                    panic!("❗️❗️❗️ Too many retries, failed to fetch all blocks")
                }
                let fetched_blocks_clone = blocks_map.read().await.clone();
                let blocks_to_fetch: Vec<u64> = blocks
                    .iter()
                    .filter(|block_number| !fetched_blocks_clone.contains(*block_number))
                    .take(chunk_size as usize)
                    .cloned()
                    .collect();

                let fetch_futures = blocks_to_fetch.into_iter().map(|block_number| {
                    let fetched_blocks_clone = blocks_map.clone();
                    let rpc_sender = rpc_sender.clone();
                    let provider_clone = provider_clone.clone();

                    async move {
                        let storage_proof = provider_clone
                            .get_proof(address, vec![storage_key])
                            .block_id(block_number.into())
                            .await;
                        match storage_proof {
                            Ok(storage_proof) => {
                                let mut blocks_identifier = fetched_blocks_clone.write().await;
                                rpc_sender
                                    .send((block_number, storage_proof))
                                    .await
                                    .unwrap();
                                blocks_identifier.insert(block_number);
                            }
                            Err(e) => {
                                if let Some(backoff) = handle_error(e) {
                                    let mut delay = backoff;
                                    while delay <= 4 {
                                        tokio::time::sleep(Duration::from_nanos(delay)).await;
                                        delay *= 2;
                                    }
                                }
                            }
                        }
                    }
                });

                join_all(fetch_futures).await;
            }
        });
    }
}

fn handle_error(e: RpcError<TransportErrorKind>) -> Option<u64> {
    match e {
        RpcError::Transport(transport_error) => match transport_error {
            TransportErrorKind::HttpError(http_error) => {
                if http_error.status == 429 {
                    Some(1) // Start backoff with 1 milisecond
                } else {
                    None
                }
            }
            TransportErrorKind::MissingBatchResponse(_) => None,
            TransportErrorKind::BackendGone => None,
            TransportErrorKind::PubsubUnavailable => None,
            TransportErrorKind::Custom(_) => None,
            _ => None,
        },
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use alloy::primitives::{address, b256, B256, U256};

    use super::*;

    // Non-paid personal alchemy endpoint
    const SEPOLIA_RPC_URL: &str =
        "https://eth-sepolia.g.alchemy.com/v2/a-w72ZvoUS0dfMD_LBPAuRzHOlQEhi_m";

    #[tokio::test]
    async fn test_get_100_range_storage_with_proof_by_storage_key() {
        let start_time = Instant::now();
        let rpc_url = Url::parse(SEPOLIA_RPC_URL).unwrap();
        let provider = RpcProvider::new(rpc_url, 100);
        let block_range_start = 6127485;
        let block_range_end = 6127584;
        let target_block_range = (block_range_start..=block_range_end).collect::<Vec<u64>>();
        let target_address = address!("75CeC1db9dCeb703200EAa6595f66885C962B920");
        let target_key = b256!("3c2b98cf472a02b84793a789af8876a73167e29a1a4f8bdbcb51dbfef0a75d7b");
        let result = provider
            .get_storage_proofs(target_block_range, target_address, target_key)
            .await;
        assert!(result.is_ok());
        let result = result.unwrap();
        let length = result.len();
        assert_eq!(length, 100);
        let value = result.get(&6127485).unwrap();
        assert_eq!(value.storage_proof[0].key.0, target_key);
        assert_eq!(value.storage_proof[0].value, U256::from(20000000000000u64));
        let duration = start_time.elapsed();
        println!("Time taken (Storage Fetch): {:?}", duration);
    }

    #[tokio::test]
    async fn test_get_100_range_storage_with_proof_by_storage_slot() {
        let start_time = Instant::now();
        let rpc_url = Url::parse(SEPOLIA_RPC_URL).unwrap();
        let provider = RpcProvider::new(rpc_url, 100);
        let block_range_start = 6127485;
        let block_range_end = 6127584;
        let target_block_range = (block_range_start..=block_range_end).collect::<Vec<u64>>();
        let target_address = address!("75CeC1db9dCeb703200EAa6595f66885C962B920");
        let target_slot = B256::from(U256::from(1));
        let result = provider
            .get_storage_proofs(target_block_range, target_address, target_slot)
            .await;
        assert!(result.is_ok());
        let result = result.unwrap();
        let length = result.len();
        assert_eq!(length, 100);
        let value = result.get(&6127485).unwrap();
        assert_eq!(value.storage_proof[0].key.0, target_slot);
        assert_eq!(value.storage_proof[0].value, U256::from(20000000000000u64));
        let duration = start_time.elapsed();
        println!("Time taken (Storage Fetch): {:?}", duration);
    }

    #[tokio::test]
    async fn test_get_100_range_account_with_proof() {
        let start_time = Instant::now();
        let rpc_url = Url::parse(SEPOLIA_RPC_URL).unwrap();
        let provider = RpcProvider::new(rpc_url, 100);
        let block_range_start = 6127485;
        let block_range_end = 6127584;
        let target_block_range = (block_range_start..=block_range_end).collect::<Vec<u64>>();
        let target_address = address!("7f2c6f930306d3aa736b3a6c6a98f512f74036d4");

        let result = provider
            .get_account_proofs(target_block_range, target_address)
            .await;
        assert!(result.is_ok());
        let length = result.unwrap().len();
        assert_eq!(length, 100);
        let duration = start_time.elapsed();
        println!("Time taken (Account Fetch): {:?}", duration);
    }
}
