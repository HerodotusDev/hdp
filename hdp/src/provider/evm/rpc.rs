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
use thiserror::Error;
use tokio::sync::{
    mpsc::{self, Sender},
    RwLock,
};
use tracing::debug;

/// Error from [`RpcProvider`]
#[derive(Error, Debug)]
pub enum RpcProviderError {
    #[error("Failed to send proofs with mpsc")]
    MpscError(
        #[from]
        tokio::sync::mpsc::error::SendError<(
            BlockNumber,
            alloy::rpc::types::EIP1186AccountProofResponse,
        )>,
    ),
}

/// RPC provider for fetching data from Ethereum RPC
/// It is a wrapper around the alloy provider, using eth_getProof for fetching account and storage proofs
///
/// How to use:
/// ```rust
/// use reqwest::Url;
/// use hdp::provider::evm::rpc::RpcProvider;
/// use alloy::primitives::Address;
///
/// async fn call_provider(url: Url, chunk_size: u64, block_range_start: u64, block_range_end: u64, increment: u64, address: Address) {
///         let provider = RpcProvider::new(url, chunk_size);
///         let target_block_range = (block_range_start..=block_range_end).collect::<Vec<u64>>();
///         let result = provider.get_account_proofs(target_block_range, address).await;
///         match result {
///             Ok(proofs) => println!("Fetched proofs: {:?}", proofs),
///             Err(e) => eprintln!("Error fetching proofs: {:?}", e),
///         }
/// }
/// ```
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
        blocks: Vec<BlockNumber>,
        address: Address,
    ) -> Result<HashMap<BlockNumber, EIP1186AccountProofResponse>, RpcProviderError> {
        self.get_proofs(blocks, address, None).await
    }

    /// Get storage with proof in given vector of blocks and slot
    pub async fn get_storage_proofs(
        &self,
        block_range: Vec<BlockNumber>,
        address: Address,
        storage_key: StorageKey,
    ) -> Result<HashMap<BlockNumber, EIP1186AccountProofResponse>, RpcProviderError> {
        self.get_proofs(block_range, address, Some(storage_key))
            .await
    }

    /// Generalized function to get proofs (account or storage) in given vector of blocks
    async fn get_proofs(
        &self,
        blocks: Vec<BlockNumber>,
        address: Address,
        storage_key: Option<StorageKey>,
    ) -> Result<HashMap<BlockNumber, EIP1186AccountProofResponse>, RpcProviderError> {
        let start_fetch = Instant::now();

        let (rpc_sender, mut rx) = mpsc::channel::<(BlockNumber, EIP1186AccountProofResponse)>(32);
        self.spawn_proof_fetcher(rpc_sender, blocks, address, storage_key);

        let mut fetched_proofs = HashMap::new();
        while let Some((block_number, proof)) = rx.recv().await {
            fetched_proofs.insert(block_number, proof);
        }
        let duration = start_fetch.elapsed();
        debug!("time taken (Fetch): {:?}", duration);

        Ok(fetched_proofs)
    }

    /// Spawns a task to fetch proofs (account or storage) in parallel with chunk size
    fn spawn_proof_fetcher(
        &self,
        rpc_sender: Sender<(BlockNumber, EIP1186AccountProofResponse)>,
        blocks: Vec<BlockNumber>,
        address: Address,
        storage_key: Option<StorageKey>,
    ) {
        let chunk_size = self.chunk_size;
        let provider_clone = self.provider.clone();
        let target_blocks_length = blocks.len();

        debug!(
            "fetching proofs for {}, with chunk size: {}",
            address, chunk_size
        );

        tokio::spawn(async move {
            let mut try_count = 0;
            let blocks_map = Arc::new(RwLock::new(HashSet::<BlockNumber>::new()));

            while blocks_map.read().await.len() < target_blocks_length {
                try_count += 1;
                if try_count > 50 {
                    panic!("❗️❗️❗️ Too many retries, failed to fetch all blocks")
                }
                let fetched_blocks_clone = blocks_map.read().await.clone();

                let blocks_to_fetch: Vec<BlockNumber> = blocks
                    .iter()
                    .filter(|block_number| !fetched_blocks_clone.contains(block_number))
                    .take(chunk_size as usize)
                    .cloned()
                    .collect();

                let fetch_futures = blocks_to_fetch
                    .into_iter()
                    .map(|block_number| {
                        let fetched_blocks_clone = blocks_map.clone();
                        let rpc_sender = rpc_sender.clone();
                        let provider_clone = provider_clone.clone();
                        async move {
                            let proof =
                                fetch_proof(&provider_clone, address, block_number, storage_key)
                                    .await;
                            handle_proof_result(
                                proof,
                                block_number,
                                fetched_blocks_clone,
                                rpc_sender,
                            )
                            .await;
                        }
                    })
                    .collect::<Vec<_>>();

                join_all(fetch_futures).await;
            }
        });
    }
}

/// Fetches proof (account or storage) for a given block number
async fn fetch_proof(
    provider: &RootProvider<Http<Client>>,
    address: Address,
    block_number: BlockNumber,
    storage_key: Option<StorageKey>,
) -> Result<EIP1186AccountProofResponse, RpcError<TransportErrorKind>> {
    match storage_key {
        Some(key) => {
            provider
                .get_proof(address, vec![key])
                .block_id(block_number.into())
                .await
        }
        None => {
            provider
                .get_proof(address, vec![])
                .block_id(block_number.into())
                .await
        }
    }
}

/// Handles the result of a proof fetch operation
async fn handle_proof_result(
    proof: Result<EIP1186AccountProofResponse, RpcError<TransportErrorKind>>,
    block_number: BlockNumber,
    blocks_map: Arc<RwLock<HashSet<BlockNumber>>>,
    rpc_sender: Sender<(BlockNumber, EIP1186AccountProofResponse)>,
) {
    match proof {
        Ok(proof) => {
            let mut blocks_identifier = blocks_map.write().await;
            rpc_sender
                .send((block_number, proof))
                .await
                .map_err(RpcProviderError::MpscError)
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

fn handle_error(e: RpcError<TransportErrorKind>) -> Option<u64> {
    match e {
        RpcError::Transport(TransportErrorKind::HttpError(http_error))
            if http_error.status == 429 =>
        {
            Some(1)
        }

        _ => None,
    }
}

#[cfg(test)]
#[cfg(feature = "test_utils")]
mod tests {
    use super::*;
    use crate::provider::evm::provider::EvmProvider;
    use alloy::primitives::{address, b256, B256, U256};
    use dotenv::dotenv;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn initialize() {
        INIT.call_once(|| {
            dotenv().ok();
        });
    }

    #[tokio::test]
    async fn test_get_100_range_storage_with_proof_by_storage_key() {
        initialize();
        let start_time = Instant::now();
        let provider = EvmProvider::default().rpc_provider;
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
        initialize();
        let start_time = Instant::now();
        let provider = EvmProvider::default().rpc_provider;
        let block_range_start = 6127485;
        let block_range_end = 6127584;
        let target_block_range =
            (block_range_start..=block_range_end).collect::<Vec<BlockNumber>>();
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
        initialize();
        let start_time = Instant::now();
        let provider = EvmProvider::default().rpc_provider;
        let block_range_start = 6127485;
        let block_range_end = 6127584;
        let target_block_range =
            (block_range_start..=block_range_end).collect::<Vec<BlockNumber>>();
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
