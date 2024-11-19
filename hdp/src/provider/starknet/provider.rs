use alloy::primitives::BlockNumber;
use itertools::Itertools;
#[cfg(feature = "test_utils")]
use reqwest::Url;
#[cfg(feature = "test_utils")]
use std::str::FromStr;
use std::{collections::HashMap, time::Instant};

use starknet_types_core::felt::Felt;
use tracing::info;

use crate::provider::{
    config::ProviderConfig,
    error::ProviderError,
    indexer::Indexer,
    traits::{AsyncResult, FetchProofsFromKeysResult, FetchProofsResult, ProofProvider},
};

use super::{rpc::RpcProvider, types::GetProofOutput};

type StorageProofsResult = Result<HashMap<BlockNumber, GetProofOutput>, ProviderError>;

pub struct StarknetProvider {
    /// Account and storage trie provider
    pub(crate) rpc_provider: RpcProvider,
    /// Header provider
    //TODO: indexer is not supported for starknet yet
    pub(crate) header_provider: Indexer,
}

#[cfg(feature = "test_utils")]
impl Default for StarknetProvider {
    fn default() -> Self {
        Self::new(&ProviderConfig {
            provider_url: Url::from_str("https://pathfinder.sepolia.iosis.tech/").unwrap(),
            chain_id: crate::primitives::ChainId::StarknetSepolia,
            deployed_on_chain_id: crate::primitives::ChainId::EthereumSepolia,
            max_requests: 100,
        })
    }
}

impl StarknetProvider {
    pub fn new(config: &ProviderConfig) -> Self {
        let rpc_provider = RpcProvider::new(config.provider_url.to_owned(), config.max_requests);
        // TODO: for now starknet is only supported on staging environmnet
        let indexer = Indexer::new(config.chain_id, config.deployed_on_chain_id).staging();
        Self {
            rpc_provider,
            header_provider: indexer,
        }
    }

    /// Fetches the storage proofs for the given block range.
    /// The storage proofs are fetched from the RPC provider.
    ///
    /// Return:
    /// - Storage proofs mapped by block number
    pub async fn get_range_of_storage_proofs(
        &self,
        from_block: BlockNumber,
        to_block: BlockNumber,
        increment: u64,
        address: Felt,
        storage_slot: Felt,
    ) -> StorageProofsResult {
        let start_fetch = Instant::now();

        let target_blocks_batch: Vec<Vec<BlockNumber>> =
            self._chunk_block_range(from_block, to_block, increment);

        let mut processed_accounts = HashMap::new();
        for target_blocks in target_blocks_batch {
            processed_accounts.extend(
                self.rpc_provider
                    .get_storage_proofs(target_blocks, address, vec![storage_slot])
                    .await?,
            );
        }

        let duration = start_fetch.elapsed();
        info!("time taken (Storage Proofs Fetch): {:?}", duration);

        Ok(processed_accounts)
    }

    /// Chunks the block range into smaller ranges of 800 blocks.
    /// This is to avoid fetching too many blocks at once from the RPC provider.
    /// This is meant to use with data lake definition, which have sequential block numbers
    pub(crate) fn _chunk_block_range(
        &self,
        from_block: BlockNumber,
        to_block: BlockNumber,
        increment: u64,
    ) -> Vec<Vec<BlockNumber>> {
        (from_block..=to_block)
            .step_by(increment as usize)
            .chunks(800)
            .into_iter()
            .map(|chunk| chunk.collect())
            .collect()
    }

    /// Chunks the blocks range into smaller ranges of 800 blocks.
    /// It simply consider the number of blocks in the range and divide it by 800.
    /// This is targeted for account and storage proofs in optimized way
    pub(crate) fn _chunk_vec_blocks_keys(
        &self,
        blocks: Vec<(BlockNumber, Vec<Felt>)>,
    ) -> Vec<Vec<(BlockNumber, Vec<Felt>)>> {
        blocks.chunks(800).map(|chunk| chunk.to_vec()).collect()
    }

    /// Chunks the blocks into smaller ranges of 800 blocks.
    /// This is targeted for indexer to fetch header proofs in optimized way
    pub(crate) fn _chunk_vec_blocks_for_indexer(
        &self,
        blocks: Vec<BlockNumber>,
    ) -> Vec<Vec<BlockNumber>> {
        // Sort the blocks
        let mut sorted_blocks = blocks.clone();
        sorted_blocks.sort();

        let mut result: Vec<Vec<BlockNumber>> = Vec::new();
        let mut current_chunk: Vec<BlockNumber> = Vec::new();

        for &block in sorted_blocks.iter() {
            // Check if the current chunk is empty or if the difference is within 800 blocks
            if current_chunk.is_empty() || block - current_chunk[0] <= 800 {
                current_chunk.push(block);
            } else {
                // Push the current chunk to result and start a new chunk
                result.push(current_chunk);
                current_chunk = vec![block];
            }
        }

        if !current_chunk.is_empty() {
            result.push(current_chunk);
        }

        result
    }
}

impl ProofProvider for StarknetProvider {
    // TODO: it will be later deprecated with datalake deprecation
    fn fetch_proofs<'a>(
        &'a self,
        _datalake: &'a crate::primitives::task::datalake::DatalakeCompute,
    ) -> AsyncResult<FetchProofsResult> {
        unimplemented!("fetch_proofs is not implemented for StarknetProvider");
    }

    fn fetch_proofs_from_keys(
        &self,
        keys: crate::provider::key::CategorizedFetchKeys,
    ) -> AsyncResult<FetchProofsFromKeysResult> {
        Box::pin(async move { self.fetch_proofs_from_keys(keys).await })
    }
}
