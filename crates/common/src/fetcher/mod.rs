use anyhow::Result;
use std::{collections::HashMap, time::Instant};

use crate::{
    block::{account::Account, header::BlockHeader},
    datalake::base::MMRMetaResult,
};

use self::{
    memory::{MemoryFetcher, RlpEncodedValue, StoredHeader, StoredHeaders},
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

    /// Fetches the headers of the blocks and relevant MMR metatdata in the given block range.
    /// return a tuple of the headers hashmap and the MMR metadata.
    pub async fn get_full_header_with_proof(
        &mut self,
        block_numbers: Vec<u64>,
    ) -> Result<(StoredHeaders, MMRMetaResult)> {
        //? A map of block numbers to a boolean indicating whether the block was fetched.
        let mut blocks_map: HashMap<u64, (bool, StoredHeader)> = HashMap::new();
        // TODO: in v0 we assume all the blocks in data lake are exist in 1 MMR
        let mut mmr_assume_for_now = MMRMetaResult {
            mmr_id: 0,
            mmr_peaks: vec![],
            mmr_root: "".to_string(),
            mmr_size: 0,
        };

        // 1. Fetch headers from memory
        for block_number in &block_numbers {
            let header = self.memory.get_full_header_with_proof(*block_number);
            if let Some(fetched_header) = header {
                blocks_map.insert(*block_number, (true, fetched_header));
            }
        }

        // construct blocknumbers list doesn't exist in memeory
        let mut block_numbers_to_fetch_from_indexer: Vec<u64> = vec![];
        for block_number in &block_numbers {
            if !blocks_map.contains_key(block_number) {
                block_numbers_to_fetch_from_indexer.push(*block_number);
            }
        }

        // 2. Fetch MMR data and header data from Herodotus indexer
        let start_fetch = Instant::now();
        let indexer_fetcher =
            RpcFetcher::new("https://rs-indexer.api.herodotus.cloud/accumulators".to_string());
        let mmr_data = indexer_fetcher
            .get_mmr_from_indexer(&block_numbers_to_fetch_from_indexer)
            .await;
        match mmr_data {
            Ok(mmr) => {
                println!("✅ Successfully fetched MMR data from indexer");
                let duration = start_fetch.elapsed();
                println!("Time taken (fetch from Indexer): {:?}", duration);
                // update blocks_map with the fetched data from indexer
                for block_number in &block_numbers {
                    if let Some(header) = mmr.1.get(block_number) {
                        let mmr_meta = &mmr.0;
                        // set retrieved MMR to memory
                        self.memory.set_mmr_data(
                            mmr_meta.mmr_id,
                            mmr_meta.mmr_root.clone(),
                            mmr_meta.mmr_size,
                            mmr_meta.mmr_peaks.clone(),
                        );
                        mmr_assume_for_now = MMRMetaResult {
                            mmr_id: mmr_meta.mmr_id,
                            mmr_peaks: mmr_meta.mmr_peaks.clone(),
                            mmr_root: mmr_meta.mmr_root.clone(),
                            mmr_size: mmr_meta.mmr_size,
                        };
                        blocks_map.insert(
                            *block_number,
                            (
                                true,
                                (
                                    header.rlp_block_header.clone(),
                                    header.siblings_hashes.clone(),
                                    header.element_index,
                                    mmr_meta.mmr_id,
                                ),
                            ),
                        );
                    }
                }
            }
            Err(e) => {
                let duration = start_fetch.elapsed();
                println!("Time taken (during from Indexer): {:?}", duration);
                println!(
                    "❌ Something went wrong while fetching MMR data from indexer: {}",
                    e
                );
            }
        }

        // format into Vec<StoredHeaders>
        let mut stored_headers: StoredHeaders = HashMap::new();
        blocks_map
            .iter()
            .for_each(|(block_number, (fetched, header))| {
                if *fetched {
                    stored_headers.insert(*block_number, header.clone());
                }
            });
        Ok((stored_headers, mmr_assume_for_now))
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

    pub async fn get_account_with_proof(
        &mut self,
        block_number: u64,
        account: String,
    ) -> (String, Vec<String>) {
        match self.memory.get_account(block_number, account.clone()) {
            Some(account_with_proof) => account_with_proof,
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
                    account_proof.clone(),
                );
                (rlp_encoded, account_proof)
            }
        }
    }

    pub async fn get_storage_value_with_proof(
        &mut self,
        block_number: u64,
        account: String,
        slot: String,
    ) -> (String, Vec<String>) {
        match self
            .memory
            .get_storage(block_number, account.clone(), slot.clone())
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
                    storage_proof.clone(),
                );
                (storage_value, storage_proof)
            }
        }
    }
}
