use futures::future::join_all;
use std::{collections::HashMap, sync::Arc, time::Instant};
use tokio::sync::RwLock;

use crate::block::{account::Account, header::BlockHeader};

use self::{
    memory::{MemoryFetcher, RlpEncodedValue},
    rpc::RpcFetcher,
};

pub mod memory;
pub mod prefilled_data;
pub mod rpc;

/// `AbstractFetcher` abstracts the fetching of data from the RPC and memory.
///  It uses a `MemoryFetcher` and a `RpcFetcher` to fetch data.
///
/// TODO: Lock only rpc fetcher and keep the memory fetcher unlocked
/// but handle requests so that it would not make duplicate requests
pub struct AbstractFetcher {
    memory: MemoryFetcher,
    rpc: RpcFetcher,
}

pub struct ChunkHeaderResult {
    pub headers: Vec<BlockHeader>,
    pub missing_blocks: Vec<u64>,
}

impl AbstractFetcher {
    pub fn new(rpc_url: String) -> Self {
        Self {
            memory: MemoryFetcher::new(),
            rpc: RpcFetcher::new(rpc_url),
        }
    }

    pub async fn get_rlp_headers(&mut self, block_numbers: Vec<u64>) -> Vec<RlpEncodedValue> {
        let start_fetch = Instant::now();
        //? A map of block numbers to a boolean indicating whether the block was fetched.
        let blocks_map: Arc<RwLock<HashMap<u64, (bool, RlpEncodedValue)>>> =
            Arc::new(RwLock::new(HashMap::new()));

        for block_number in &block_numbers {
            let header = self.memory.get_rlp_header_with_proof(*block_number);
            if let Some((rlp_header, _, _, _)) = header {
                let mut blocks_map_write = blocks_map.write().await;
                blocks_map_write.insert(*block_number, (true, rlp_header));
            }
        }

        // Prepare and execute futures for missing blocks
        let fetch_futures: Vec<_> = block_numbers
            .clone()
            .into_iter()
            .map(|block_number| {
                let blocks_map_clone = blocks_map.clone();
                let rpc_clone = self.rpc.clone(); // Assume self.rpc can be cloned or is wrapped in Arc
                async move {
                    let should_fetch = {
                        let read_guard = blocks_map_clone.read().await;
                        !read_guard.contains_key(&block_number)
                    };
                    if should_fetch {
                        match rpc_clone.get_block_by_number(block_number).await {
                            Ok(header_rpc) => {
                                let mut write_guard = blocks_map_clone.write().await;
                                let block_header = BlockHeader::from(&header_rpc);
                                let rlp_encoded = block_header.rlp_encode();
                                write_guard.insert(block_number, (true, rlp_encoded));
                                // Assuming conversion to RlpEncodedValue is done here
                            }
                            Err(e) => {
                                eprintln!("Failed to fetch block {}: {}", block_number, e);
                                // Optionally handle errors by inserting a placeholder or error indicator
                            }
                        }
                    }
                }
            })
            .collect();

        // Execute fetch operations in parallel
        join_all(fetch_futures).await;
        // Construct the final result vector from blocks_map
        let blocks_map_read = blocks_map.read().await;
        let duration = start_fetch.elapsed();
        println!("✅ Successfully fetched headers from rpc");
        println!("Time taken (fetch headers from rpc): {:?}", duration);

        // Fetch MMR data from Herodotus indexer
        let indexer_fetcher = RpcFetcher::new("https://ds-indexer.api.herodotus.cloud".to_string());
        let mmr_data = indexer_fetcher
            .get_mmr_from_indexer(&block_numbers)
            .await
            .unwrap();
        println!("✅ Successfully fetched MMR data from indexer");
        println!("mmr_data: {:?}", mmr_data);

        // Cache the MMR data and header in memory
        block_numbers
            .into_iter()
            .filter_map(|block_number| {
                // Get the RLP header and MMR data from the RPC call
                let rlp_header = blocks_map_read
                    .get(&block_number)
                    .map(|(_, rlp_encoded)| rlp_encoded.clone());

                // Get the MMR proof from the indexer
                // TODO: Handle in proper way later with new endpoint
                let proof = mmr_data.get(&block_number).cloned().unwrap();

                if let Some(ref header) = rlp_header {
                    // Cache the header data in memory
                    self.memory.set_header_with_proof(
                        block_number,
                        header.to_string(),
                        proof.siblings_hashes,
                        proof.element_index,
                        proof.tree_id,
                    );
                }

                // Cache the MMR data in memory
                self.memory.set_mmr_data(
                    proof.tree_id,
                    //TODO: Need to get the root from the indexer ( update endpoint to get root )
                    "0x00001".to_string(),
                    //TODO: Need to get the mmr size from the indexer ( update endpoint to get size )
                    proof.last_pos,
                    proof.peaks_hashes,
                );

                rlp_header
            })
            .collect()
    }

    // Unoptimized version of get_rlp_header, just for testing purposes
    pub async fn get_rlp_header(&mut self, block_number: u64) -> RlpEncodedValue {
        match self.memory.get_rlp_header(block_number) {
            Some(header) => header,
            None => {
                let header_rpc = self.rpc.get_block_by_number(block_number).await.unwrap();
                let block_header = BlockHeader::from(&header_rpc);
                let rlp_encoded = block_header.rlp_encode();
                self.memory.set_header(block_number, rlp_encoded.clone());

                rlp_encoded
            }
        }
    }

    pub async fn get_rlp_account(&mut self, block_number: u64, account: String) -> RlpEncodedValue {
        match self.memory.get_rlp_account(block_number, account.clone()) {
            Some(account) => account,
            None => {
                let account_rpc = self
                    .rpc
                    .get_proof(block_number, account.clone(), None)
                    .await
                    .unwrap();
                let retrieved_account = Account::from(&account_rpc);
                let rlp_encoded = retrieved_account.rlp_encode();
                let account_proof = account_rpc.account_proof;
                self.memory.set_account(
                    block_number,
                    account_rpc.address,
                    rlp_encoded.clone(),
                    account_proof,
                );
                rlp_encoded
            }
        }
    }

    pub async fn get_storage_value(
        &mut self,
        block_number: u64,
        account: String,
        slot: String,
    ) -> String {
        match self
            .memory
            .get_storage_value(block_number, account.clone(), slot.clone())
        {
            Some(storage) => storage,
            None => {
                let account_rpc = self
                    .rpc
                    .get_proof(block_number, account.clone(), Some(vec![slot.clone()]))
                    .await
                    .unwrap();
                let retrieved_account = Account::from(&account_rpc);
                let rlp_encoded = retrieved_account.rlp_encode();
                let storage = &account_rpc.storage_proof[0];
                let storage_slot = storage.key.clone();
                let storage_value = storage.value.clone();
                let storage_proof = account_rpc.storage_proof[0].proof.clone();
                self.memory.set_storage(
                    block_number,
                    account.clone(),
                    rlp_encoded,
                    account_rpc.account_proof,
                    storage_slot,
                    storage_value.clone(),
                    storage_proof,
                );
                storage_value
            }
        }
    }
}
