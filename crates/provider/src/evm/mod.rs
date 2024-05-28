use alloy_primitives::Bytes;
use anyhow::Result;
use eth_trie_proofs::{tx_receipt_trie::TxReceiptsMptHandler, tx_trie::TxsMptHandler};
use std::{collections::HashMap, time::Instant};
use tokio::sync::mpsc;
use tracing::{error, info};

use hdp_primitives::{block::header::Header, datalake::output::MMRMeta};

use self::{
    memory::{InMemoryProvider, RlpEncodedValue, StoredHeader, StoredHeaders},
    rpc::{FetchedAccountProof, FetchedStorageProof, HeaderProvider, TrieProofProvider},
};

pub(crate) mod memory;
pub(crate) mod rpc;

// For more information swagger doc: https://rs-indexer.api.herodotus.cloud/swagger
const HERODOTUS_RS_INDEXER_URL: &str = "https://rs-indexer.api.herodotus.cloud/accumulators";

/// [`AbstractProvider`] abstracts the fetching of data from the RPC and memory.
///  It uses a [`InMemoryProvider`] and a [`RpcProvider`] to fetch data.
///
/// TODO: Optimization idea, Lock only rpc provider and keep the memory provider unlocked
/// but handle requests so that it would not make duplicate requests
pub struct AbstractProvider {
    /// [`InMemoryProvider`] is used to fetch data from memory.
    // TODO: It's not using for now
    memory: InMemoryProvider,
    /// Fetch data from the RPC
    trie_proof_provider: TrieProofProvider,
    /// Fetch block headers and MMR data from the Herodotus indexer.
    header_provider: HeaderProvider,
}

impl AbstractProvider {
    pub fn new(rpc_url: &'static str, chain_id: u64, rpc_chunk_size: u64) -> Self {
        Self {
            memory: InMemoryProvider::new(),
            trie_proof_provider: TrieProofProvider::new(rpc_url, rpc_chunk_size),
            header_provider: HeaderProvider::new(HERODOTUS_RS_INDEXER_URL, chain_id),
        }
    }

    // TODO: wip
    pub async fn get_sequencial_full_header_with_proof(
        &self,
        start_block: u64,
        end_block: u64,
    ) -> Result<(StoredHeaders, MMRMeta)> {
        //? A map of block numbers to a boolean indicating whether the block was fetched.
        let mut blocks_map: HashMap<u64, StoredHeader> = HashMap::new();

        // Fetch MMR data and header data from Herodotus indexer
        let start_fetch = Instant::now();

        let mmr_data = self
            .header_provider
            .get_sequencial_headers_and_mmr_from_indexer(start_block, end_block)
            .await;

        match mmr_data {
            Ok(mmr) => {
                info!("Successfully fetched MMR data from indexer");
                let duration = start_fetch.elapsed();
                info!("Time taken (fetch from Indexer): {:?}", duration);
                for block_proof in &mmr.1 {
                    blocks_map.insert(
                        *block_proof.0,
                        (
                            block_proof.1.rlp_block_header.value.clone(),
                            block_proof.1.siblings_hashes.clone(),
                            block_proof.1.element_index,
                            mmr.0.mmr_id,
                        ),
                    );
                }

                Ok((
                    blocks_map,
                    MMRMeta {
                        id: mmr.0.mmr_id,
                        root: mmr.0.mmr_root,
                        size: mmr.0.mmr_size,
                        peaks: mmr.0.mmr_peaks,
                    },
                ))
            }
            Err(e) => {
                let duration = start_fetch.elapsed();
                info!("Time taken (during from Indexer): {:?}", duration);
                error!(
                    "Something went wrong while fetching MMR data from indexer: {}",
                    e
                );
                Err(e)
            }
        }
    }

    // /// Fetches the headers of the blocks and relevant MMR metatdata in the given block range.
    // /// return a tuple of the headers hashmap and the MMR metadata.
    // // THIS ENDPOINT IS NOT USED, but we need this approach if introduce in memory cache
    // pub async fn get_full_header_with_proof(
    //     &mut self,
    //     block_numbers: Vec<u64>,
    // ) -> Result<(StoredHeaders, MMRMeta)> {
    //     //? A map of block numbers to a boolean indicating whether the block was fetched.
    //     let mut blocks_map: HashMap<u64, (bool, StoredHeader)> = HashMap::new();

    //     let mut relevant_mmr: HashSet<MMRMeta> = HashSet::new();

    //     // 1. Fetch headers from memory
    //     for block_number in &block_numbers {
    //         let header = self.memory.get_full_header_with_proof(*block_number);
    //         if let Some(fetched_header) = header {
    //             blocks_map.insert(*block_number, (true, fetched_header));
    //         }
    //     }

    //     // construct blocknumbers list doesn't exist in memeory
    //     let mut block_numbers_to_fetch_from_indexer: Vec<u64> = vec![];
    //     for block_number in &block_numbers {
    //         if !blocks_map.contains_key(block_number) {
    //             block_numbers_to_fetch_from_indexer.push(*block_number);
    //         }
    //     }

    //     // 2. Fetch MMR data and header data from Herodotus indexer
    //     let start_fetch = Instant::now();

    //     let mmr_data = self
    //         .header_provider
    //         .get_mmr_from_indexer(&block_numbers_to_fetch_from_indexer)
    //         .await;
    //     match mmr_data {
    //         Ok(mmr) => {
    //             info!("Successfully fetched MMR data from indexer");
    //             let duration = start_fetch.elapsed();
    //             info!("Time taken (fetch from Indexer): {:?}", duration);
    //             // update blocks_map with the fetched data from indexer
    //             for block_number in &block_numbers {
    //                 if let Some(header) = mmr.1.get(block_number) {
    //                     let mmr_meta = &mmr.0;
    //                     // set retrieved MMR to memory
    //                     self.memory.set_mmr_data(
    //                         mmr_meta.mmr_id,
    //                         mmr_meta.mmr_root.clone(),
    //                         mmr_meta.mmr_size,
    //                         mmr_meta.mmr_peaks.clone(),
    //                     );

    //                     relevant_mmr.insert(MMRMeta {
    //                         id: mmr_meta.mmr_id,
    //                         root: mmr_meta.mmr_root.clone(),
    //                         size: mmr_meta.mmr_size,
    //                         peaks: mmr_meta.mmr_peaks.clone(),
    //                     });

    //                     blocks_map.insert(
    //                         *block_number,
    //                         (
    //                             true,
    //                             (
    //                                 header.rlp_block_header.clone(),
    //                                 header.siblings_hashes.clone(),
    //                                 header.element_index,
    //                                 mmr_meta.mmr_id,
    //                             ),
    //                         ),
    //                     );
    //                 }
    //             }
    //         }
    //         Err(e) => {
    //             let duration = start_fetch.elapsed();
    //             info!("Time taken (during from Indexer): {:?}", duration);
    //             error!(
    //                 "Something went wrong while fetching MMR data from indexer: {}",
    //                 e
    //             );
    //             return Err(e);
    //         }
    //     }

    //     // format into Vec<StoredHeaders>
    //     let mut stored_headers: StoredHeaders = HashMap::new();
    //     blocks_map
    //         .iter()
    //         .for_each(|(block_number, (fetched, header))| {
    //             if *fetched {
    //                 stored_headers.insert(*block_number, header.clone());
    //             }
    //         });

    //     // TODO: in v1 allowed to handle all the blocks in datalake are exist in 1 MMR
    //     let mmr_meta_result = match relevant_mmr.len() {
    //         0 => None,
    //         1 => relevant_mmr.iter().next().cloned(),
    //         _ => relevant_mmr.iter().next().cloned(),
    //     };

    //     match mmr_meta_result {
    //         Some(mmr_meta) => Ok((stored_headers, mmr_meta)),
    //         None => bail!("No MMR metadata found"),
    //     }
    // }

    // Unoptimized version of get_rlp_header, just for testing purposes
    pub async fn get_rlp_header(&mut self, block_number: u64) -> RlpEncodedValue {
        match self.memory.get_rlp_header(block_number) {
            Some(header) => header,
            None => {
                let header_rpc = self
                    .trie_proof_provider
                    .get_block_by_number(block_number)
                    .await
                    .unwrap();
                let block_header = Header::from(&header_rpc);
                let rlp_encoded = block_header.rlp_encode();
                self.memory.set_header(block_number, rlp_encoded.clone());

                rlp_encoded
            }
        }
    }

    // Get account with proof in given range of blocks
    // This need to be used for block sampled datalake
    pub async fn get_range_account_with_proof(
        &mut self,
        block_range_start: u64,
        block_range_end: u64,
        increment: u64,
        address: String,
    ) -> Result<HashMap<u64, FetchedAccountProof>> {
        let start_fetch = Instant::now();

        let target_block_range: Vec<u64> = (block_range_start..=block_range_end)
            .step_by(increment as usize)
            .collect();

        let (rpc_sender, mut rx) = mpsc::channel::<FetchedAccountProof>(32);

        self.trie_proof_provider
            .get_account_proofs(rpc_sender, target_block_range, &address)
            .await;

        let mut result = HashMap::new();

        while let Some(proof) = rx.recv().await {
            result.insert(proof.block_number, proof);
        }

        let duration = start_fetch.elapsed();
        info!("Time taken (Account Fetch): {:?}", duration);

        Ok(result)
    }

    // Get storage with proof in given range of blocks
    // This need to be used for block sampled datalake
    pub async fn get_range_storage_with_proof(
        &mut self,
        block_range_start: u64,
        block_range_end: u64,
        increment: u64,
        address: String,
        slot: String,
    ) -> Result<HashMap<u64, FetchedStorageProof>> {
        let start_fetch = Instant::now();
        //? A map of block numbers to a boolean indicating whether the block was fetched.
        let target_block_range: Vec<u64> = (block_range_start..=block_range_end)
            .step_by(increment as usize)
            .collect();

        let (rpc_sender, mut rx) = mpsc::channel::<FetchedStorageProof>(32);
        self.trie_proof_provider
            .get_storage_proofs(rpc_sender, target_block_range, &address, slot)
            .await;

        let mut result = HashMap::new();

        while let Some(proof) = rx.recv().await {
            result.insert(proof.block_number, proof);
        }
        let duration = start_fetch.elapsed();
        info!("Time taken (Storage Fetch): {:?}", duration);

        Ok(result)
    }

    /// Fetches the encoded transaction with proof from the MPT trie for the given block number.
    /// The transaction is fetched from the MPT trie and the proof is generated from the MPT trie.
    pub async fn get_tx_with_proof_from_block(
        &self,
        target_block: u64,
        start_index: u64,
        end_index: u64,
        incremental: u64,
    ) -> Result<Vec<(u64, u64, String, Vec<String>, u8)>> {
        let mut tx_with_proof = vec![];
        let mut txs_mpt_handler = TxsMptHandler::new(self.trie_proof_provider.url).unwrap();
        txs_mpt_handler
            .build_tx_tree_from_block(target_block)
            .await
            .unwrap();
        let txs = txs_mpt_handler.get_elements().unwrap();

        let target_tx_index_range = (start_index..end_index).step_by(incremental as usize);
        for tx_index in target_tx_index_range {
            let proof = txs_mpt_handler
                .get_proof(tx_index)
                .unwrap()
                .into_iter()
                .map(|x| Bytes::from(x).to_string())
                .collect::<Vec<_>>();
            let consensus_tx = txs[tx_index as usize].clone();
            let rlp = Bytes::from(consensus_tx.rlp_encode()).to_string();
            let tx_type = consensus_tx.0.tx_type() as u8;
            tx_with_proof.push((target_block, tx_index, rlp, proof, tx_type));
        }

        Ok(tx_with_proof)
    }

    /// Fetches the encoded transaction receipt with proof from the MPT trie for the given block number.
    /// The transaction receipt is fetched from the MPT trie and the proof is generated from the MPT trie.
    pub async fn get_tx_receipt_with_proof_from_block(
        &self,
        target_block: u64,
        start_index: u64,
        end_index: u64,
        incremental: u64,
    ) -> Result<Vec<(u64, u64, String, Vec<String>, u8)>> {
        let mut tx_receipt_with_proof = vec![];
        let mut tx_reciepts_mpt_handler =
            TxReceiptsMptHandler::new(self.trie_proof_provider.url).unwrap();

        tx_reciepts_mpt_handler
            .build_tx_receipts_tree_from_block(target_block)
            .await
            .unwrap();
        let tx_receipts = tx_reciepts_mpt_handler.get_elements().unwrap();
        let target_tx_receipt_index_range = (start_index..end_index).step_by(incremental as usize);

        for tx_receipt_index in target_tx_receipt_index_range {
            let proof = tx_reciepts_mpt_handler
                .get_proof(tx_receipt_index)
                .unwrap()
                .into_iter()
                .map(|x| Bytes::from(x).to_string())
                .collect::<Vec<_>>();
            let consensus_tx_receipt = tx_receipts[tx_receipt_index as usize].clone();
            let rlp = Bytes::from(consensus_tx_receipt.rlp_encode()).to_string();
            let tx_receipt_type = consensus_tx_receipt.0.tx_type() as u8;
            tx_receipt_with_proof.push((
                target_block,
                tx_receipt_index,
                rlp,
                proof,
                tx_receipt_type,
            ));
        }

        Ok(tx_receipt_with_proof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{hex, keccak256};

    fn rlp_string_to_block_hash(rlp_string: &str) -> String {
        keccak256(hex::decode(rlp_string).unwrap()).to_string()
    }

    // Non-paid personal alchemy endpoint
    const SEPOLIA_RPC_URL: &str =
        "https://eth-sepolia.g.alchemy.com/v2/xar76cftwEtqTBWdF4ZFy9n8FLHAETDv";

    #[tokio::test]
    async fn test_provider_get_rlp_header() {
        let mut provider = AbstractProvider::new(SEPOLIA_RPC_URL, 11155111, 40);
        let rlp_header = provider.get_rlp_header(0).await;
        let block_hash = rlp_string_to_block_hash(&rlp_header);
        assert_eq!(
            block_hash,
            "0x25a5cc106eea7138acab33231d7160d69cb777ee0c2c553fcddf5138993e6dd9"
        );
        let rlp_header = provider.get_rlp_header(5521772).await;
        let block_hash = rlp_string_to_block_hash(&rlp_header);
        assert_eq!(
            block_hash,
            "0xe72515bc74912f67912a64a458e6f2cd2742f8dfe0666e985749483dab0b7b9a"
        );
        let rlp_header = provider.get_rlp_header(487680).await;
        let block_hash = rlp_string_to_block_hash(&rlp_header);
        assert_eq!(
            block_hash,
            "0xf494127d30817d04b634eae9f6139d8155ee4c78ba60a35bd7be187378e93d6e"
        );
    }

    #[tokio::test]
    async fn get_block_range_from_nonce_range_non_constant() {
        let provider = AbstractProvider::new(SEPOLIA_RPC_URL, 11155111, 40);
        let block_range = provider
            .get_tx_with_proof_from_block(5530433, 10, 100, 1)
            .await
            .unwrap();

        assert_eq!(block_range.len(), 90);
        assert_eq!(block_range[0].2,"0xf873830beeeb84faa6fd50830148209447b854ad2ddb01cfee0b07f4e2da0ac50277b1168806f05b59d3b20000808401546d72a06af2b103dfb7bccc757d575bc11c38f2ecd1a22ca2fcf95a602119582c607927a047329735997e3357dfd7d63eda024d35f7012855aa12ba210f9ed311a517b5e6");

        let block_range = provider
            .get_tx_with_proof_from_block(5530433, 10, 100, 3)
            .await
            .unwrap();

        assert_eq!(block_range.len(), 30);
    }
}
