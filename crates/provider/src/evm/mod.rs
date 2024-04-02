use anyhow::{bail, Result};
use core::panic;
use futures::future::join_all;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{join, sync::RwLock, task};
use tracing::{error, info};

use hdp_primitives::{
    block::{account::Account, header::Header},
    datalake::block_sampled::types::MMRMeta,
};

use self::{
    memory::{InMemoryProvider, RlpEncodedValue, StoredHeader, StoredHeaders},
    rpc::RpcProvider,
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
#[derive(Clone)]
pub struct AbstractProvider {
    /// [`InMemoryProvider`] is used to fetch data from memory.
    // TODO: It's not using for now
    memory: InMemoryProvider,
    /// Fetch data from the RPC for account and storage.
    account_provider: RpcProvider,
    /// Fetch block headers and MMR data from the Herodotus indexer.
    header_provider: RpcProvider,
}

impl AbstractProvider {
    pub fn new(rpc_url: &'static str, chain_id: u64) -> Self {
        Self {
            memory: InMemoryProvider::new(),
            account_provider: RpcProvider::new(rpc_url, chain_id),
            header_provider: RpcProvider::new(HERODOTUS_RS_INDEXER_URL, chain_id),
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
                    .account_provider
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

    // pub async fn get_account_with_proof(
    //     &mut self,
    //     block_number: u64,
    //     account: &str,
    // ) -> (String, Vec<String>) {
    //     match self.memory.get_account(block_number, account) {
    //         Some(account_with_proof) => account_with_proof,
    //         None => {
    //             let account_rpc = self
    //                 .account_provider
    //                 .get_proof(block_number, account, None)
    //                 .await
    //                 .unwrap();
    //             let retrieved_account = Account::from(&account_rpc);
    //             let rlp_encoded = retrieved_account.rlp_encode();
    //             let account_proof = account_rpc.account_proof;
    //             self.memory.set_account(
    //                 block_number,
    //                 account_rpc.address,
    //                 rlp_encoded.clone(),
    //                 account_proof.clone(),
    //             );
    //             (rlp_encoded, account_proof)
    //         }
    //     }
    // }

    // Get account with proof in given range of blocks
    // This need to be used for block sampled datalake
    pub async fn get_range_account_with_proof(
        &mut self,
        block_range_start: u64,
        block_range_end: u64,
        increment: u64,
        address: String,
    ) -> Result<HashMap<u64, (String, Vec<String>)>> {
        let start_fetch = Instant::now();
        //? A map of block numbers to a boolean indicating whether the block was fetched.
        // This contains rlp encoded account and account proof
        let blocks_map = Arc::new(RwLock::new(HashMap::new()));

        let target_block_range: Vec<u64> = (block_range_start..=block_range_end)
            .step_by(increment as usize)
            .collect();

        for block_number in &target_block_range {
            let account = self.memory.get_account(*block_number, &address);
            if let Some(account) = account {
                let mut blocks_map_write = blocks_map.write().await;
                blocks_map_write.insert(*block_number, (true, account.0, account.1));
            }
        }

        // chunk the target_block_range into 50
        let target_block_range_chunks: Vec<Vec<u64>> =
            target_block_range.chunks(50).map(|x| x.to_vec()).collect();

        for one_chunked_block_range in target_block_range_chunks {
            // Prepare and execute futures for missing blocks
            let fetch_futures: Vec<_> = one_chunked_block_range
                .clone()
                .into_iter()
                .map(|block_number| {
                    let blocks_map_clone = blocks_map.clone();
                    let rpc_clone = self.account_provider.clone(); // Assume self.rpc can be cloned or is wrapped in Arc
                    let address_clone = address.clone();
                    async move {
                        let should_fetch = {
                            let read_guard = blocks_map_clone.read().await;
                            !read_guard.contains_key(&block_number)
                        };
                        if should_fetch {
                            match rpc_clone
                                .get_proof(block_number, &address_clone, None)
                                .await
                            {
                                Ok(account_from_rpc) => {
                                    let mut write_guard = blocks_map_clone.write().await;
                                    let retrieved_account = Account::from(&account_from_rpc);
                                    let rlp_encoded_account = retrieved_account.rlp_encode();
                                    let account_proof = account_from_rpc.account_proof;
                                    write_guard.insert(
                                        block_number,
                                        (true, rlp_encoded_account, account_proof),
                                    );
                                    // Assuming conversion to RlpEncodedValue is done here
                                }
                                Err(e) => {
                                    error!(
                                        "Failed to fetch account in block {}: {}",
                                        block_number, e
                                    );
                                    bail!(e);
                                    // Optionally handle errors by inserting a placeholder or error indicator
                                }
                            }
                        }
                        Ok(())
                    }
                })
                .collect();

            // Execute fetch operations in parallel
            join_all(fetch_futures).await;
        }

        // Construct the final result vector from blocks_map
        let blocks_map_read = blocks_map.read().await;
        let duration = start_fetch.elapsed();
        info!("Time taken (Account Fetch): {:?}", duration);

        let mut result = HashMap::new();

        for block in &target_block_range {
            if !blocks_map_read.contains_key(block) {
                bail!("Failed to fetch account in block {}", block);
            }
            result.insert(
                *block,
                blocks_map_read
                    .get(block)
                    .map(|(_, b, c)| (b.clone(), c.clone()))
                    .unwrap(),
            );
        }

        Ok(result)
    }

    // pub async fn get_storage_value_with_proof(
    //     &mut self,
    //     block_number: u64,
    //     account: String,
    //     slot: String,
    // ) -> Result<(String, Vec<String>)> {
    //     match self
    //         .memory
    //         .get_storage(block_number, account.clone(), slot.clone())
    //     {
    //         Some(storage) => Ok(storage),
    //         None => {
    //             let account_rpc = self
    //                 .account_provider
    //                 .get_proof(block_number, &account, Some(vec![slot.clone()]))
    //                 .await?;
    //             let retrieved_account = Account::from(&account_rpc);
    //             let rlp_encoded = retrieved_account.rlp_encode();
    //             let storage = &account_rpc.storage_proof[0];
    //             let storage_slot = storage.key.clone();
    //             let storage_value = storage.value.clone();
    //             let storage_proof = account_rpc.storage_proof[0].proof.clone();
    //             self.memory.set_storage(
    //                 block_number,
    //                 account.clone(),
    //                 rlp_encoded,
    //                 account_rpc.account_proof,
    //                 storage_slot,
    //                 storage_value.clone(),
    //                 storage_proof.clone(),
    //             );
    //             Ok((storage_value, storage_proof))
    //         }
    //     }
    // }

    // Get storage with proof in given range of blocks
    // This need to be used for block sampled datalake
    pub async fn get_range_storage_with_proof(
        &mut self,
        block_range_start: u64,
        block_range_end: u64,
        increment: u64,
        address: String,
        slot: String,
    ) -> Result<HashMap<u64, (String, Vec<String>, String, Vec<String>)>> {
        let start_fetch = Instant::now();
        //? A map of block numbers to a boolean indicating whether the block was fetched.
        // This contains rlp encoded account, account proof, storage value and storage proof
        let blocks_map = Arc::new(RwLock::new(HashMap::new()));

        let target_block_range: Vec<u64> = (block_range_start..=block_range_end)
            .step_by(increment as usize)
            .collect();

        for block_number in &target_block_range {
            let storage = self
                .memory
                .get_storage(*block_number, address.clone(), slot.clone());
            let account = self.memory.get_account(*block_number, &address);
            if let Some(storage) = storage {
                let retrieved_account = account.unwrap();
                let mut blocks_map_write = blocks_map.write().await;
                blocks_map_write.insert(
                    *block_number,
                    (
                        true,
                        retrieved_account.0,
                        retrieved_account.1,
                        storage.0,
                        storage.1,
                    ),
                );
            }
        }

        // chunk the target_block_range into 50
        let target_block_range_chunks: Vec<Vec<u64>> =
            target_block_range.chunks(50).map(|x| x.to_vec()).collect();

        for one_chunk_block_range in target_block_range_chunks {
            // Prepare and execute futures for missing blocks
            let fetch_futures: Vec<_> = one_chunk_block_range
                .clone()
                .into_iter()
                .map(|block_number| {
                    let blocks_map_clone = blocks_map.clone();
                    let rpc_clone = self.account_provider.clone(); // Assume self.rpc can be cloned or is wrapped in Arc
                    let address_clone = address.clone();
                    let slot_clone = slot.clone();
                    async move {
                        let should_fetch = {
                            let read_guard = blocks_map_clone.read().await;
                            !read_guard.contains_key(&block_number)
                        };
                        if should_fetch {
                            match rpc_clone
                                .get_proof(block_number, &address_clone, Some(vec![slot_clone]))
                                .await
                            {
                                Ok(account_from_rpc) => {
                                    let mut write_guard = blocks_map_clone.write().await;
                                    let retrieved_account = Account::from(&account_from_rpc);
                                    let storage = &account_from_rpc.storage_proof[0];
                                    let rlp_encoded = retrieved_account.rlp_encode();
                                    let storage_value = storage.value.clone();
                                    let storage_proof =
                                        account_from_rpc.storage_proof[0].proof.clone();
                                    let account_proof = account_from_rpc.account_proof;
                                    write_guard.insert(
                                        block_number,
                                        (
                                            true,
                                            rlp_encoded,
                                            account_proof,
                                            storage_value,
                                            storage_proof,
                                        ),
                                    );
                                    // Assuming conversion to RlpEncodedValue is done here
                                }
                                Err(e) => {
                                    // TODO: handle error in proper way
                                    if e.to_string().contains("No storage proof found") {
                                        error!("Storage value not exist: {}", e);
                                        panic!("{}", e);
                                    } else {
                                        error!(
                                            "Failed to fetch storage in block {}: {}",
                                            block_number, e
                                        );
                                        bail!(e);
                                    }
                                }
                            }
                        }
                        Ok(())
                    }
                })
                .collect();

            // Execute fetch operations in parallel
            join_all(fetch_futures).await;
        }

        // Construct the final result vector from blocks_map
        let blocks_map_read = blocks_map.read().await;
        let duration = start_fetch.elapsed();
        info!("Time taken (Storage Fetch): {:?}", duration);

        let mut result = HashMap::new();

        for block in &target_block_range {
            if !blocks_map_read.contains_key(block) {
                bail!("Failed to fetch storage in block {}", block);
            }
            result.insert(
                *block,
                blocks_map_read
                    .get(block)
                    .map(|(_, a, b, c, d)| (a.clone(), b.clone(), c.clone(), d.clone()))
                    .unwrap(),
            );
        }

        Ok(result)
    }

    pub async fn get_block_range_from_nonce_range(
        &self,
        start_nonce: u64,
        end_nonce: u64,
        incremental: u64,
        sender: String,
    ) -> Result<Vec<u64>> {
        // `eth_getTransactionCount` to loop through blocks and find the block range by nonce range
        let latest_block_number = self.account_provider.get_latest_block_number().await?;

        // Perform binary searches in parallel
        let (start_block_result, end_block_result) = join!(
            self.binary_search_for_nonce(start_nonce, &sender, 0, latest_block_number),
            self.binary_search_for_nonce(end_nonce, &sender, 0, latest_block_number)
        );

        let start_block = start_block_result?;
        let end_block = end_block_result?;

        println!("✅start_block: {}", start_block);
        println!("✅end_block: {}", end_block);

        let target_nonce_range: Vec<u64> = (start_nonce..=end_nonce)
            .step_by(incremental as usize)
            .collect();

        if target_nonce_range.len() > 2 {
            // Slicing the vector to skip the first and the last elements, then converting to a vector
            let target_nonce_range_sliced =
                target_nonce_range[1..target_nonce_range.len() - 1].to_vec();

            // chunk the target_nonce_range into 5
            let target_nonce_range_chunks: Vec<Vec<u64>> = target_nonce_range_sliced
                .chunks(5)
                .map(|x| x.to_vec())
                .collect();
            println!("✅target_nonce_range: {:?}", target_nonce_range);
            // use the genesis block number as the lower bound
            let lower_bound = start_block;
            // use the latest block number as the upper bound
            let upper_bound = end_block - 1;
            let self_arc = Arc::new(self.clone());
            // TODO: chunk the target_nonce_range into 5(?) and perform binary searches concurrently
            // TODO: loop through the chunks and update the lower and upper bounds
            let mut results = vec![];
            for one_chunked_nonce_range in target_nonce_range_chunks {
                let futures: Vec<_> = one_chunked_nonce_range
                    .into_iter()
                    .map(|nonce| {
                        let self_clone = Arc::clone(&self_arc);
                        let sender_clone = sender.clone();

                        task::spawn(async move {
                            self_clone
                                .binary_search_for_nonce(
                                    nonce,
                                    &sender_clone,
                                    lower_bound,
                                    upper_bound,
                                )
                                .await
                        })
                    })
                    .collect();

                let chunked_results: Vec<u64> = join_all(futures)
                    .await
                    .into_iter()
                    .map(|res| res.unwrap().unwrap())
                    .collect();
                println!("✅results: {:?}", chunked_results);
                results.extend(chunked_results);
            }

            // insert the start and end block numbers to the results
            results.insert(0, start_block);
            results.push(end_block);

            Ok(results)
        } else {
            Ok(vec![start_block, end_block])
        }
    }

    async fn binary_search_for_nonce(
        &self,
        target_nonce: u64,
        sender: &str,
        lower_bound: u64,
        upper_bound: u64,
    ) -> Result<u64> {
        let mut total_duration = Duration::new(0, 0);
        let mut total_rpc_call = 0;
        let mut inner_lower_bound = lower_bound;
        let mut inner_upper_bound = upper_bound;

        while inner_lower_bound <= inner_upper_bound {
            let mid = (inner_lower_bound + inner_upper_bound) / 2;
            let start_fetch = Instant::now();
            let mid_nonce = self
                .account_provider
                .get_transaction_count(sender, mid)
                .await
                .unwrap();
            let duration = start_fetch.elapsed();
            info!("Time taken (tx_count): {:?}", duration);
            total_duration += duration;
            total_rpc_call += 1;

            #[allow(clippy::comparison_chain)]
            if mid_nonce == target_nonce {
                let start_fetch = Instant::now();
                let next_block_nonce = self
                    .account_provider
                    .get_transaction_count(sender, mid + 1)
                    .await?;
                let duration = start_fetch.elapsed();
                info!("Time taken (tx_count): {:?}", duration);
                total_duration += duration;
                total_rpc_call += 1;

                // This is indeed transition
                if next_block_nonce == target_nonce + 1 {
                    // mid + 1 is the block number of first transition
                    return Ok(mid + 1);
                } else {
                    // This means index is in range of target nonces
                    inner_lower_bound = mid + 1;
                }
            } else if mid_nonce < target_nonce {
                inner_lower_bound = mid + 1;
            } else if mid_nonce > target_nonce {
                inner_upper_bound = mid - 1;
            }

            // println!("target: {}", target_nonce);
            // println!("🔍mid: {}", mid);
            // println!("🔍mid_nonce: {}", mid_nonce);
            // println!("🔍inner_lower_bound: {}", inner_lower_bound);
            // println!("🔍inner_upper_bound: {}", inner_upper_bound);
        }

        println!("😌target_nonce: {}", target_nonce);
        println!("inner_lower_bound: {}", inner_lower_bound);
        println!("inner_upper_bound: {}", inner_upper_bound);

        println!(
            "🕒total_duration: {:?} for {:?} rpc calls",
            total_duration, total_rpc_call
        );

        Ok(inner_lower_bound)
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

    const SEPOLIA_TARGET_ADDRESS: &str = "0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4";

    #[tokio::test]
    async fn test_provider_get_rlp_header() {
        let mut provider = AbstractProvider::new(SEPOLIA_RPC_URL, 11155111);
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

    // Both test works, but if call concurrently with other tests, CI will fails
    // #[tokio::test]
    // async fn get_block_range_from_massive_nonce_range() {
    //     let provider = AbstractProvider::new(SEPOLIA_RPC_URL, 11155111);
    //     let block_range = provider
    //         .get_block_range_from_nonce_range(63878, 63898, 1, SEPOLIA_TARGET_ADDRESS.to_string())
    //         .await
    //         .unwrap();
    //     assert_eq!(
    //         block_range,
    //         vec![
    //             5604974, 5604986, 5604994, 5605004, 5605015, 5605024, 5605034, 5605044, 5605054,
    //             5605064, 5605075, 5605084, 5605094, 5605104, 5605114, 5605127, 5605134, 5605145,
    //             5605154, 5605164, 5605174
    //         ]
    //     );
    // }

    // #[tokio::test]
    // async fn get_block_range_from_nonce_range() {
    //     let provider = AbstractProvider::new(SEPOLIA_RPC_URL, 11155111);
    //     let block_range = provider
    //         .get_block_range_from_nonce_range(63878, 63888, 1, SEPOLIA_TARGET_ADDRESS.to_string())
    //         .await
    //         .unwrap();
    //     assert_eq!(
    //         block_range,
    //         vec![
    //             5604974, 5604986, 5604994, 5605004, 5605015, 5605024, 5605034, 5605044, 5605054,
    //             5605064, 5605075
    //         ]
    //     );
    // }

    #[tokio::test]
    async fn get_block_range_from_nonce_smol_range() {
        // The range is doable with non chunked approach
        let provider = AbstractProvider::new(SEPOLIA_RPC_URL, 11155111);

        let block_range = provider
            .get_block_range_from_nonce_range(63878, 63885, 1, SEPOLIA_TARGET_ADDRESS.to_string())
            .await
            .unwrap();
        assert_eq!(
            block_range,
            vec![5604974, 5604986, 5604994, 5605004, 5605015, 5605024, 5605034, 5605044]
        );
    }

    // const SEPOLIA_TARGET_ADDRESS_NON_CONSTANT: &str = "0x0a4De450feB156A2A51eD159b2fb99Da26E5F3A3";

    // #[tokio::test]
    // async fn get_block_range_from_nonce_range_non_constant() {
    //     let provider = AbstractProvider::new(SEPOLIA_RPC_URL, 11155111);
    //     let block_range = provider
    //         .get_block_range_from_nonce_range(
    //             520,
    //             524,
    //             1,
    //             SEPOLIA_TARGET_ADDRESS_NON_CONSTANT.to_string(),
    //         )
    //         .await
    //         .unwrap();
    //     assert_eq!(
    //         block_range,
    //         vec![5530433, 5530441, 5530878, 5556642, 5572347]
    //     );
    // }
}
