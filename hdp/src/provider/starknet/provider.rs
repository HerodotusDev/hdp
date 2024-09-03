use std::{collections::HashMap, time::Instant};

use alloy::primitives::BlockNumber;
use itertools::Itertools;
use starknet_types_core::felt::Felt;
use tracing::info;

use crate::provider::{config::ProviderConfig, error::ProviderError, indexer::Indexer};

use super::{rpc::RpcProvider, types::GetProofOutput};

type AccountProofsResult = Result<HashMap<BlockNumber, GetProofOutput>, ProviderError>;
type StorageProofsResult = Result<HashMap<BlockNumber, GetProofOutput>, ProviderError>;

pub struct StarknetProvider {
    /// Account and storage trie provider
    pub(crate) rpc_provider: RpcProvider,
    /// Header provider
    //TODO: indexer is not supported for starknet yet
    pub(crate) _header_provider: Indexer,
}

#[cfg(feature = "test_utils")]
impl Default for StarknetProvider {
    fn default() -> Self {
        Self::new(&ProviderConfig::default())
    }
}

impl StarknetProvider {
    pub fn new(config: &ProviderConfig) -> Self {
        let rpc_provider = RpcProvider::new(config.rpc_url.to_owned(), config.max_requests);
        let indexer = Indexer::new(config.chain_id);
        Self {
            rpc_provider,
            _header_provider: indexer,
        }
    }

    /// Fetches the account proofs for the given block range.
    /// The account proofs are fetched from the RPC provider.
    ///
    /// Return:
    /// - Account proofs mapped by block number
    pub async fn get_range_of_account_proofs(
        &self,
        from_block: BlockNumber,
        to_block: BlockNumber,
        increment: u64,
        address: Felt,
    ) -> AccountProofsResult {
        let start_fetch = Instant::now();

        let target_blocks_batch: Vec<Vec<BlockNumber>> =
            self._chunk_block_range(from_block, to_block, increment);

        let mut fetched_accounts_proofs_with_blocks_map = HashMap::new();
        for target_blocks in target_blocks_batch {
            fetched_accounts_proofs_with_blocks_map.extend(
                self.rpc_provider
                    .get_account_proofs(target_blocks, address)
                    .await?,
            );
        }

        let duration = start_fetch.elapsed();
        info!("time taken (Account Proofs Fetch): {:?}", duration);

        Ok(fetched_accounts_proofs_with_blocks_map)
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
                    .get_storage_proofs(target_blocks, address, storage_slot)
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
}
