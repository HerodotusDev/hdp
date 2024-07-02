use alloy::{
    primitives::{Address, BlockNumber, Bytes, ChainId, StorageKey, TxIndex},
    rpc::types::EIP1186AccountProofResponse,
    transports::{RpcError, TransportErrorKind},
};
use eth_trie_proofs::{
    tx_receipt_trie::TxReceiptsMptHandler, tx_trie::TxsMptHandler, EthTrieError,
};
use hdp_primitives::block::header::{MMRMetaFromNewIndexer, MMRProofFromNewIndexer};
use itertools::Itertools;
use reqwest::Url;
use std::{collections::HashMap, time::Instant};
use thiserror::Error;
use tracing::info;

use crate::{
    indexer::{Indexer, IndexerError},
    types::{FetchedTransactionProof, FetchedTransactionReceiptProof},
};

use super::rpc::{RpcProvider, RpcProviderError};

/// This is optimal max number of requests to send in parallel when using non-paid alchemy rpc url
const DEFAULT_MAX_REQUESTS: u64 = 100;

/// Error from [`EvmProvider`]
#[derive(Error, Debug)]
pub enum ProviderError {
    /// Error when the query is invalid
    #[error("Transaction index out of bound: requested index: {0}, length: {1}")]
    OutOfBoundRequestError(u64, u64),

    /// Error when the MMR meta is mismatched among range of requested blocks
    #[error("MMR meta mismatch among range of requested blocks")]
    MismatchedMMRMeta,

    /// Error from the [`Indexer`]
    #[error("Failed from indexer")]
    IndexerError(#[from] IndexerError),

    /// Error from [`RpcProvider`]
    #[error("Failed to get proofs: {0}")]
    RpcProviderError(#[from] RpcProviderError),

    /// Error from [`eth_trie_proofs`]
    #[error("EthTrieError: {0}")]
    EthTrieError(#[from] eth_trie_proofs::EthTrieError),

    #[error("Fetch key error: {0}")]
    FetchKeyError(String),
}

/// EVM provider
///
/// This provider is responsible for fetching proofs from the EVM chain.
/// It uses the RPC provider to fetch proofs from the EVM chain and the indexer to fetch
/// header proofs
///
/// Run benchmark [here](../benchmark/provider_benchmark.rs)
#[derive(Clone)]
pub struct EvmProvider {
    /// Account and storage trie provider
    pub(crate) rpc_provider: super::rpc::RpcProvider,
    /// Header provider
    pub(crate) header_provider: Indexer,
    /// transaction url
    pub(crate) tx_provider_url: Url,
}

/// EVM provider configuration
#[derive(Clone, Debug)]
pub struct EvmProviderConfig {
    /// RPC url
    pub rpc_url: Url,
    /// Chain id
    pub chain_id: u64,
    /// Max number of requests to send in parallel
    ///
    /// For default, it is set to 100
    /// For archive node, recommend to set it to 1000
    /// This will effect fetch speed of account, storage proofs
    pub max_requests: u64,
}

impl EvmProvider {
    pub fn new(config: EvmProviderConfig) -> Self {
        let rpc_provider = RpcProvider::new(config.rpc_url.clone(), config.max_requests);
        let header_provider = Indexer::new(config.chain_id);

        Self {
            rpc_provider,
            header_provider,
            tx_provider_url: config.rpc_url,
        }
    }

    pub fn new_with_url(url: Url, chain_id: ChainId) -> Self {
        let rpc_provider = RpcProvider::new(url.clone(), DEFAULT_MAX_REQUESTS);
        let header_provider = Indexer::new(chain_id);

        Self {
            rpc_provider,
            header_provider,
            tx_provider_url: url,
        }
    }

    /// Fetches the header proofs for the given block range.
    /// The header proofs are fetched from the indexer and the MMR meta is fetched from the indexer.
    ///
    /// Return:
    /// - MMR meta
    /// - Header proofs mapped by block number
    pub async fn get_range_of_header_proofs(
        &self,
        from_block: BlockNumber,
        to_block: BlockNumber,
        increment: u64,
    ) -> Result<
        (
            MMRMetaFromNewIndexer,
            HashMap<BlockNumber, MMRProofFromNewIndexer>,
        ),
        ProviderError,
    > {
        let start_fetch = Instant::now();

        let target_blocks_batch: Vec<Vec<BlockNumber>> =
            self._chunk_block_range(from_block, to_block, increment);

        let mut fetched_headers_proofs_with_blocks_map = HashMap::new();
        let mut mmr = None;

        for target_blocks in target_blocks_batch {
            let (start_block, end_block) =
                (target_blocks[0], target_blocks[target_blocks.len() - 1]);

            let indexer_response = self
                .header_provider
                .get_headers_proof(start_block, end_block)
                .await?;

            // validate MMR among range of blocks
            match mmr {
                None => {
                    mmr = Some(indexer_response.mmr_meta);
                }
                Some(ref existing_mmr) if existing_mmr != &indexer_response.mmr_meta => {
                    return Err(ProviderError::MismatchedMMRMeta);
                }
                _ => {}
            }
            fetched_headers_proofs_with_blocks_map.extend(indexer_response.headers);
        }

        let duration = start_fetch.elapsed();
        info!("Time taken (Headers Proofs Fetch): {:?}", duration);

        Ok((mmr.unwrap(), fetched_headers_proofs_with_blocks_map))
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
        address: Address,
    ) -> Result<HashMap<BlockNumber, EIP1186AccountProofResponse>, ProviderError> {
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
        info!("Time taken (Account Proofs Fetch): {:?}", duration);

        Ok(fetched_accounts_proofs_with_blocks_map)
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
    pub(crate) fn _chunk_vec_blocks_for_mpt(
        &self,
        blocks: Vec<BlockNumber>,
    ) -> Vec<Vec<BlockNumber>> {
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
        address: Address,
        storage_slot: StorageKey,
    ) -> Result<HashMap<BlockNumber, EIP1186AccountProofResponse>, ProviderError> {
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
        info!("Time taken (Storage Proofs Fetch): {:?}", duration);

        Ok(processed_accounts)
    }

    /// Fetches the encoded transaction with proof from the MPT trie for the given block number.
    /// The transaction is fetched from the MPT trie and the proof is generated from the MPT trie.
    ///
    /// Return:
    /// - Transaction proofs mapped by block number
    pub async fn get_tx_with_proof_from_block(
        &self,
        target_block: BlockNumber,
        start_index: TxIndex,
        end_index: TxIndex,
        incremental: u64,
    ) -> Result<Vec<FetchedTransactionProof>, ProviderError> {
        let start_fetch = Instant::now();

        let mut fetched_transaction_proofs = vec![];
        let mut tx_trie_provider = TxsMptHandler::new(self.tx_provider_url.clone()).unwrap();

        loop {
            let trie_response = tx_trie_provider
                .build_tx_tree_from_block(target_block)
                .await;

            match trie_response {
                Ok(_) => break,
                Err(EthTrieError::RPC(RpcError::Transport(TransportErrorKind::HttpError(
                    http_error,
                )))) if http_error.status == 429 => {
                    // retry if 429 error
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    continue;
                }
                _ => return Err(ProviderError::EthTrieError(trie_response.err().unwrap())),
            }
        }

        let fetched_transactions = tx_trie_provider.get_elements()?;
        let tx_length = fetched_transactions.len() as u64;
        let target_tx_index_range = (start_index..end_index).step_by(incremental as usize);
        for tx_index in target_tx_index_range {
            // validate out of bound request
            if tx_index >= tx_length {
                return Err(ProviderError::OutOfBoundRequestError(tx_index, tx_length));
            }

            let tx_trie_proof = tx_trie_provider
                .get_proof(tx_index)
                .unwrap()
                .into_iter()
                .map(Bytes::from)
                .collect::<Vec<_>>();

            let consensus_tx = fetched_transactions[tx_index as usize].clone();
            fetched_transaction_proofs.push(FetchedTransactionProof::new(
                target_block,
                tx_index,
                consensus_tx.rlp_encode(),
                tx_trie_proof,
                consensus_tx.0.tx_type(),
            ));
        }

        let duration = start_fetch.elapsed();
        info!("Time taken (Transactions Proofs Fetch): {:?}", duration);

        Ok(fetched_transaction_proofs)
    }

    /// Fetches the transaction receipts with proof from the MPT trie for the given block number.
    /// The transaction receipts are fetched from the MPT trie and the proof is generated from the MPT trie.
    ///
    /// Return:
    /// - Transaction receipts proofs mapped by block number
    pub async fn get_tx_receipt_with_proof_from_block(
        &self,
        target_block: BlockNumber,
        start_index: TxIndex,
        end_index: TxIndex,
        incremental: u64,
    ) -> Result<Vec<FetchedTransactionReceiptProof>, ProviderError> {
        let start_fetch = Instant::now();

        let mut fetched_transaction_receipts_proofs = vec![];
        let mut tx_receipt_trie_provider = TxReceiptsMptHandler::new(self.tx_provider_url.clone())?;

        loop {
            let trie_response = tx_receipt_trie_provider
                .build_tx_receipts_tree_from_block(target_block)
                .await;

            match trie_response {
                Ok(_) => break,
                Err(EthTrieError::RPC(RpcError::Transport(TransportErrorKind::HttpError(
                    http_error,
                )))) if http_error.status == 429 => {
                    // retry if 429 error
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    continue;
                }
                _ => return Err(ProviderError::EthTrieError(trie_response.err().unwrap())),
            }
        }

        let fetched_transaction_receipts = tx_receipt_trie_provider.get_elements()?;
        let tx_receipt_length = fetched_transaction_receipts.len() as u64;
        let target_tx_index_range = (start_index..end_index).step_by(incremental as usize);
        for tx_index in target_tx_index_range {
            // validate out of bound request
            if tx_index >= tx_receipt_length {
                return Err(ProviderError::OutOfBoundRequestError(
                    tx_index,
                    tx_receipt_length,
                ));
            }

            let tx_receipt_trie_proof = tx_receipt_trie_provider
                .get_proof(tx_index)
                .unwrap()
                .into_iter()
                .map(Bytes::from)
                .collect::<Vec<_>>();

            let consensus_tx_receipt = fetched_transaction_receipts[tx_index as usize].clone();
            fetched_transaction_receipts_proofs.push(FetchedTransactionReceiptProof::new(
                target_block,
                tx_index,
                consensus_tx_receipt.rlp_encode(),
                tx_receipt_trie_proof,
                consensus_tx_receipt.0.tx_type(),
            ));
        }

        let duration = start_fetch.elapsed();
        info!(
            "Time taken (Transaction Receipts Proofs Fetch): {:?}",
            duration
        );

        Ok(fetched_transaction_receipts_proofs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{address, B256};

    const SEPOLIA_RPC_URL: &str =
        "https://eth-sepolia.g.alchemy.com/v2/xar76cftwEtqTBWdF4ZFy9n8FLHAETDv";

    #[ignore = "too many requests, recommend to run locally"]
    #[tokio::test]
    async fn test_get_2000_range_of_account_proofs() -> Result<(), ProviderError> {
        let start_time = Instant::now();
        let provider = EvmProvider::new_with_url(Url::parse(SEPOLIA_RPC_URL).unwrap(), 1155511);
        let target_address = address!("7f2c6f930306d3aa736b3a6c6a98f512f74036d4");
        let response = provider
            .get_range_of_account_proofs(6127485, 6127485 + 2000 - 1, 1, target_address)
            .await;
        assert!(response.is_ok());
        let length = response.unwrap().len();
        assert_eq!(length, 2000);
        let duration = start_time.elapsed();
        println!("Time taken (Account Fetch): {:?}", duration);
        Ok(())
    }

    #[ignore = "too many requests, recommend to run locally"]
    #[tokio::test]
    async fn test_get_2000_range_of_storage_proofs() -> Result<(), ProviderError> {
        let start_time = Instant::now();
        let provider = EvmProvider::new_with_url(Url::parse(SEPOLIA_RPC_URL).unwrap(), 11155111);
        let target_address = address!("75CeC1db9dCeb703200EAa6595f66885C962B920");
        let result = provider
            .get_range_of_storage_proofs(6127485, 6127485 + 2000 - 1, 1, target_address, B256::ZERO)
            .await;
        assert!(result.is_ok());
        let length = result.unwrap().len();
        assert_eq!(length, 2000);
        let duration = start_time.elapsed();
        println!("Time taken (Storage Fetch): {:?}", duration);
        Ok(())
    }

    #[ignore = "too many requests, recommend to run locally"]
    #[tokio::test]
    async fn test_get_2000_range_of_header_proofs() -> Result<(), ProviderError> {
        let start_time = Instant::now();
        let provider = EvmProvider::new_with_url(Url::parse(SEPOLIA_RPC_URL).unwrap(), 11155111);
        let (_meta, header_response) = provider
            .get_range_of_header_proofs(6127485, 6127485 + 2000 - 1, 1)
            .await?;
        assert_eq!(header_response.len(), 2000);
        // assert_eq!(meta.mmr_id, 26);
        let duration = start_time.elapsed();
        println!("Time taken (Header Fetch): {:?}", duration);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_parallel_4_all_tx_with_proof_from_block() {
        let provider = EvmProvider::new_with_url(Url::parse(SEPOLIA_RPC_URL).unwrap(), 11155111);

        let task1 = {
            let provider = provider.clone();
            tokio::spawn(async move {
                provider
                    .get_tx_with_proof_from_block(6127485, 0, 23, 1)
                    .await
            })
        };

        let task2 = {
            let provider = provider.clone();
            tokio::spawn(async move {
                provider
                    .get_tx_with_proof_from_block(6127486, 0, 20, 1)
                    .await
            })
        };

        let task3 = {
            let provider = provider.clone();
            tokio::spawn(async move {
                provider
                    .get_tx_with_proof_from_block(6127487, 1, 1 + 29, 1)
                    .await
            })
        };

        let task4 = {
            let provider = provider.clone();
            tokio::spawn(async move {
                provider
                    .get_tx_with_proof_from_block(6127488, 5, 5 + 75, 1)
                    .await
            })
        };

        let (result1, result2, result3, result4) =
            tokio::try_join!(task1, task2, task3, task4).unwrap();
        // validate result 1
        assert_eq!(result1.unwrap().len(), 23);
        // validate result 2
        assert_eq!(result2.unwrap().len(), 20);
        // validate result 3
        assert_eq!(result3.unwrap().len(), 29);
        // validate result 4
        assert_eq!(result4.unwrap().len(), 75);
    }

    #[tokio::test]
    async fn test_get_parallel_4_all_tx_receipt_with_proof_from_block() {
        let provider = EvmProvider::new_with_url(Url::parse(SEPOLIA_RPC_URL).unwrap(), 11155111);
        let task1 = {
            let provider = provider.clone();
            tokio::spawn(async move {
                provider
                    .get_tx_receipt_with_proof_from_block(6127485, 0, 23, 1)
                    .await
            })
        };

        let task2 = {
            let provider = provider.clone();
            tokio::spawn(async move {
                provider
                    .get_tx_receipt_with_proof_from_block(6127486, 0, 20, 1)
                    .await
            })
        };

        let task3 = {
            let provider = provider.clone();
            tokio::spawn(async move {
                provider
                    .get_tx_receipt_with_proof_from_block(6127487, 1, 30, 1)
                    .await
            })
        };

        let task4 = {
            let provider = provider.clone();
            tokio::spawn(async move {
                provider
                    .get_tx_receipt_with_proof_from_block(6127488, 5, 80, 1)
                    .await
            })
        };

        let (result1, result2, result3, result4) =
            tokio::try_join!(task1, task2, task3, task4).unwrap();

        // validate result 1
        assert_eq!(result1.unwrap().len(), 23);
        // validate result 2
        assert_eq!(result2.unwrap().len(), 20);
        // validate result 3
        assert_eq!(result3.unwrap().len(), 29);
        // validate result 4
        assert_eq!(result4.unwrap().len(), 75);
    }

    #[tokio::test]
    async fn test_error_get_tx_with_proof_from_block() {
        let provider = EvmProvider::new_with_url(Url::parse(SEPOLIA_RPC_URL).unwrap(), 11155111);
        let response = provider
            .get_tx_with_proof_from_block(6127485, 0, 2000, 1)
            .await;
        assert!(response.is_err());
        assert!(matches!(
            response,
            Err(ProviderError::OutOfBoundRequestError(93, 93))
        ));
    }

    #[tokio::test]
    async fn test_error_get_tx_receipt_with_proof_from_block() {
        let provider = EvmProvider::new_with_url(Url::parse(SEPOLIA_RPC_URL).unwrap(), 1155511);
        let response = provider
            .get_tx_receipt_with_proof_from_block(6127485, 0, 2000, 1)
            .await;
        assert!(response.is_err());
        assert!(matches!(
            response,
            Err(ProviderError::OutOfBoundRequestError(93, 93))
        ));
    }
}
