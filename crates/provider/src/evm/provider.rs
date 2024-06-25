use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

use alloy::{
    primitives::{Address, BlockNumber, Bytes, StorageKey},
    rpc::types::EIP1186AccountProofResponse,
};
use eth_trie_proofs::{tx_receipt_trie::TxReceiptsMptHandler, tx_trie::TxsMptHandler};
use hdp_primitives::{
    block::header::{MMRMetaFromNewIndexer, MMRProofFromNewIndexer},
    processed_types::block_proofs::ProcessedBlockProofs,
};
use itertools::Itertools;
use reqwest::Url;
use tracing::info;

use crate::{
    errors::ProviderError, indexer::Indexer, key::FetchKeyEnvelope, types::FetchedTransactionProof,
};

use super::rpc::RpcProvider;

pub struct EvmProvider {
    /// Account and storage trie provider
    rpc_provider: super::rpc::RpcProvider,
    /// Header provider
    header_provider: Indexer,
    /// transaction url
    tx_provider_url: Url,
}

pub struct EvmProviderConfig {
    pub rpc_url: Url,
    pub chain_id: u64,
}

impl EvmProvider {
    pub fn new(config: EvmProviderConfig) -> Self {
        let rpc_provider = RpcProvider::new(config.rpc_url.clone(), 100);
        let header_provider = Indexer::new(config.chain_id);

        Self {
            rpc_provider,
            header_provider,
            tx_provider_url: config.rpc_url,
        }
    }

    pub fn new_with_url(url: Url, chain_id: u64) -> Self {
        let rpc_provider = RpcProvider::new(url.clone(), 100);
        let header_provider = Indexer::new(chain_id);

        Self {
            rpc_provider,
            header_provider,
            tx_provider_url: url,
        }
    }

    #[allow(unused)]
    pub async fn fetch_proofs_from_keys(
        &self,
        fetch_keys: HashSet<FetchKeyEnvelope>,
    ) -> Result<ProcessedBlockProofs, ProviderError> {
        todo!("Implement fetch_proofs_from_keys")
    }

    pub async fn get_range_of_header_proofs(
        &self,
        from_block: u64,
        to_block: u64,
        increment: u64,
    ) -> Result<
        (
            MMRMetaFromNewIndexer,
            HashMap<BlockNumber, MMRProofFromNewIndexer>,
        ),
        ProviderError,
    > {
        let start_fetch = Instant::now();
        let target_blocks: Vec<Vec<u64>> = self._chunk_block_range(from_block, to_block, increment);

        let mut processed_headers = HashMap::new();
        let mut mmr = None;
        for target_block in target_blocks {
            let start_block = target_block[0];
            let end_block = target_block[target_block.len() - 1];
            let result = self
                .header_provider
                .get_headers_proof(start_block, end_block)
                .await?;
            processed_headers.extend(result.headers);

            if mmr.is_none() {
                mmr = Some(result.mmr_meta);
            } else {
                assert_eq!(mmr.as_ref().unwrap(), &result.mmr_meta);
            }
        }

        let duration = start_fetch.elapsed();
        info!("Time taken (Header Fetch): {:?}", duration);

        Ok((mmr.unwrap(), processed_headers))
    }

    pub async fn get_range_of_account_proofs(
        &self,
        from_block: u64,
        to_block: u64,
        increment: u64,
        address: Address,
    ) -> Result<HashMap<BlockNumber, EIP1186AccountProofResponse>, ProviderError> {
        let start_fetch = Instant::now();

        let target_blocks: Vec<Vec<u64>> = self._chunk_block_range(from_block, to_block, increment);

        println!("Target blocks: {:?}", target_blocks);

        let mut processed_accounts = HashMap::new();
        for target_block in target_blocks {
            let result = self
                .rpc_provider
                .get_account_proofs(target_block, address)
                .await?;
            processed_accounts.extend(result);
        }

        let duration = start_fetch.elapsed();
        info!("Time taken (Account Fetch): {:?}", duration);

        Ok(processed_accounts)
    }

    fn _chunk_block_range(&self, from_block: u64, to_block: u64, increment: u64) -> Vec<Vec<u64>> {
        (from_block..=to_block)
            .step_by(increment as usize)
            .chunks(800)
            .into_iter()
            .map(|chunk| chunk.collect())
            .collect()
    }

    pub async fn get_range_of_storage_proofs(
        &self,
        from_block: u64,
        to_block: u64,
        increment: u64,
        address: Address,
        storage_slot: StorageKey,
    ) -> Result<HashMap<BlockNumber, EIP1186AccountProofResponse>, ProviderError> {
        let start_fetch = Instant::now();
        let target_blocks: Vec<Vec<u64>> = self._chunk_block_range(from_block, to_block, increment);

        let mut processed_accounts = HashMap::new();
        for target_block in target_blocks {
            let result = self
                .rpc_provider
                .get_storage_proofs(target_block, address, storage_slot)
                .await?;
            processed_accounts.extend(result);
        }

        let duration = start_fetch.elapsed();
        info!("Time taken (Storage Fetch): {:?}", duration);
        Ok(processed_accounts)
    }

    /// Fetches the encoded transaction with proof from the MPT trie for the given block number.
    /// The transaction is fetched from the MPT trie and the proof is generated from the MPT trie.
    pub async fn get_tx_with_proof_from_block(
        &self,
        target_block: u64,
        start_index: u64,
        end_index: u64,
        incremental: u64,
    ) -> Result<Vec<FetchedTransactionProof>, ProviderError> {
        let mut tx_with_proof = vec![];
        let mut tx_trie_provider = TxsMptHandler::new(self.tx_provider_url.as_ref()).unwrap();
        tx_trie_provider
            .build_tx_tree_from_block(target_block)
            .await
            .unwrap();
        let txs = tx_trie_provider.get_elements().unwrap();

        let target_tx_index_range = (start_index..end_index).step_by(incremental as usize);
        for tx_index in target_tx_index_range {
            let proof = tx_trie_provider
                .get_proof(tx_index)
                .unwrap()
                .into_iter()
                .map(Bytes::from)
                .collect::<Vec<_>>();
            if tx_index >= txs.len() as u64 {
                return Err(ProviderError::GetTransactionProofError(format!(
                    "tx index should be less than the number of transactions {}",
                    txs.len()
                )));
            }
            let consensus_tx = txs[tx_index as usize].clone();
            let rlp = Bytes::from(consensus_tx.rlp_encode());
            let tx_type = consensus_tx.0.tx_type() as u8;
            let fetched_result = FetchedTransactionProof {
                block_number: target_block,
                tx_index,
                encoded_transaction: rlp,
                transaction_proof: proof,
                tx_type,
            };
            tx_with_proof.push(fetched_result);
        }

        Ok(tx_with_proof)
    }

    pub async fn get_tx_receipt_with_proof_from_block(
        &self,
        target_block: u64,
        start_index: u64,
        end_index: u64,
        incremental: u64,
    ) -> Result<Vec<FetchedTransactionProof>, ProviderError> {
        let mut tx_with_proof = vec![];
        let mut tx_receipt_trie_provider =
            TxReceiptsMptHandler::new(self.tx_provider_url.as_ref()).unwrap();
        tx_receipt_trie_provider
            .build_tx_receipts_tree_from_block(target_block)
            .await
            .unwrap();
        let tx_receipts = tx_receipt_trie_provider.get_elements().unwrap();

        let target_tx_index_range = (start_index..end_index).step_by(incremental as usize);
        for tx_index in target_tx_index_range {
            let proof = tx_receipt_trie_provider
                .get_proof(tx_index)
                .unwrap()
                .into_iter()
                .map(Bytes::from)
                .collect::<Vec<_>>();
            if tx_index >= tx_receipts.len() as u64 {
                return Err(ProviderError::GetTransactionReceiptProofError(format!(
                    "tx index should be less than the number of transactions receipts {}",
                    tx_receipts.len()
                )));
            }
            let consensus_tx_receipt = tx_receipts[tx_index as usize].clone();
            let rlp = Bytes::from(consensus_tx_receipt.rlp_encode());
            let tx_type = consensus_tx_receipt.0.tx_type() as u8;
            let fetched_result = FetchedTransactionProof {
                block_number: target_block,
                tx_index,
                encoded_transaction: rlp,
                transaction_proof: proof,
                tx_type,
            };
            tx_with_proof.push(fetched_result);
        }

        Ok(tx_with_proof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::{address, B256};

    const SEPOLIA_RPC_URL: &str =
        "https://eth-sepolia.g.alchemy.com/v2/xar76cftwEtqTBWdF4ZFy9n8FLHAETDv";

    #[ignore = "ignore for now"]
    #[tokio::test]
    async fn test_get_2000_range_of_account_proofs() -> Result<(), ProviderError> {
        let start_time = Instant::now();
        let provider = EvmProvider::new_with_url(Url::parse(SEPOLIA_RPC_URL).unwrap(), 1155511);
        let target_address = address!("7f2c6f930306d3aa736b3a6c6a98f512f74036d4");
        let result = provider
            .get_range_of_account_proofs(6127485, 6129484, 1, target_address)
            .await;
        assert!(result.is_ok());
        let length = result.unwrap().len();
        assert_eq!(length, 2000);
        let duration = start_time.elapsed();
        println!("Time taken (Account Fetch): {:?}", duration);
        Ok(())
    }

    #[ignore = "ignore for now"]
    #[tokio::test]
    async fn test_get_2000_range_of_storage_proofs() -> Result<(), ProviderError> {
        let start_time = Instant::now();
        let provider = EvmProvider::new_with_url(Url::parse(SEPOLIA_RPC_URL).unwrap(), 11155111);
        let target_address = address!("75CeC1db9dCeb703200EAa6595f66885C962B920");
        let result = provider
            .get_range_of_storage_proofs(6127485, 6129484, 1, target_address, B256::ZERO)
            .await;
        assert!(result.is_ok());
        let length = result.unwrap().len();
        assert_eq!(length, 2000);
        let duration = start_time.elapsed();
        println!("Time taken (Storage Fetch): {:?}", duration);
        Ok(())
    }

    #[ignore = "ignore for now"]
    #[tokio::test]
    async fn test_get_2000_range_of_header_proofs() -> Result<(), ProviderError> {
        let start_time = Instant::now();
        let provider = EvmProvider::new_with_url(Url::parse(SEPOLIA_RPC_URL).unwrap(), 11155111);
        let result = provider
            .get_range_of_header_proofs(6127485, 6129484, 1)
            .await;
        assert!(result.is_ok());
        let length = result.unwrap().1.len();
        assert_eq!(length, 2000);
        let duration = start_time.elapsed();
        println!("Time taken (Header Fetch): {:?}", duration);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_all_tx_with_proof_from_block() -> Result<(), ProviderError> {
        // TODO: tx provider cannot handle 429 error rn
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let provider = EvmProvider::new_with_url(Url::parse(SEPOLIA_RPC_URL).unwrap(), 11155111);
        let result = provider
            .get_tx_with_proof_from_block(6127485, 0, 92, 1)
            .await;
        assert!(result.is_ok());
        let length = result.unwrap().len();
        assert_eq!(length, 92);
        Ok(())
    }

    #[ignore = "ignore for now"]
    #[tokio::test]
    async fn test_get_all_tx_receipt_with_proof_from_block() -> Result<(), ProviderError> {
        // TODO: tx provider cannot handle 429 error rn
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        let provider = EvmProvider::new_with_url(Url::parse(SEPOLIA_RPC_URL).unwrap(), 11155111);
        let result = provider
            .get_tx_receipt_with_proof_from_block(6127485, 0, 92, 1)
            .await;
        assert!(result.is_ok());
        let length = result.unwrap().len();
        assert_eq!(length, 92);
        Ok(())
    }

    #[ignore = "ignore for now"]
    #[tokio::test]
    async fn test_error_get_tx_with_proof_from_block() {
        // TODO: tx provider cannot handle 429 error rn
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let provider = EvmProvider::new_with_url(Url::parse(SEPOLIA_RPC_URL).unwrap(), 11155111);
        let result = provider
            .get_tx_with_proof_from_block(6127485, 0, 2000, 1)
            .await;
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            "Failed to get transaction proof: tx index should be less than the number of transactions 93"
        );
    }

    #[tokio::test]
    async fn test_error_get_tx_receipt_with_proof_from_block() {
        // TODO: tx provider cannot handle 429 error rn
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        let provider = EvmProvider::new_with_url(Url::parse(SEPOLIA_RPC_URL).unwrap(), 1155511);
        let result = provider
            .get_tx_receipt_with_proof_from_block(6127485, 0, 2000, 1)
            .await;
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap().to_string(),
            "Failed to get transaction receipt proof: tx index should be less than the number of transactions receipts 93"
        );
    }
}
