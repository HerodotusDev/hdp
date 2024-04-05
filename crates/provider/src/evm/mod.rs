use alloy_primitives::{Bytes, B256};
use anyhow::{bail, Result};
use core::panic;
use eth_trie_proofs::tx_trie::TxsMptHandler;
use futures::future::join_all;
use std::{collections::HashMap, str::FromStr, sync::Arc, time::Instant};
use tokio::{join, sync::RwLock};
use tracing::{error, info};

use hdp_primitives::{
    block::{account::Account, header::Header},
    datalake::output::MMRMeta,
};

use self::{
    memory::{InMemoryProvider, RlpEncodedValue, StoredHeader, StoredHeaders},
    rpc::RpcProvider,
};

pub(crate) mod memory;
pub(crate) mod rpc;

// For more information swagger doc: https://rs-indexer.api.herodotus.cloud/swagger
const HERODOTUS_RS_INDEXER_URL: &str = "https://rs-indexer.api.herodotus.cloud/accumulators";
const ETHERSCAN_API: &str = "https://api.etherscan.io";
const ETHERSCAN_SEPOLIA_API: &str = "https://api-sepolia.etherscan.io";
// note: non-paid personal api key
const ETHERSCAN_API_KEY: &str = "8NTJDKIRBG8CPNQX5YGZVNGPHRYEUDTXZW";

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
    /// Fetch tx data from the Etherscan API.
    tx_provider: RpcProvider,
}

impl AbstractProvider {
    pub fn new(rpc_url: &'static str, chain_id: u64) -> Self {
        // TODO: Handle in better way
        let tx_provider = RpcProvider::new(
            if chain_id == 1 {
                ETHERSCAN_API
            } else {
                ETHERSCAN_SEPOLIA_API
            },
            chain_id,
        );

        Self {
            memory: InMemoryProvider::new(),
            account_provider: RpcProvider::new(rpc_url, chain_id),
            header_provider: RpcProvider::new(HERODOTUS_RS_INDEXER_URL, chain_id),
            tx_provider,
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

    /// Fetches the encoded transaction with proof from the given nonce range.
    /// The nonce range is inclusive.
    /// The `incremental` parameter is used to skip blocks in the range.
    pub async fn get_tx_with_proof_from_nonce_range(
        &self,
        start_nonce: u64,
        end_nonce: u64,
        incremental: u64,
        sender: String,
    ) -> Result<Vec<(u64, u64, String, Vec<String>)>> {
        let mut tx_with_proof = vec![];
        // `eth_getTransactionCount` to loop through blocks and find the block range by nonce range
        let latest_block_number = self.account_provider.get_latest_block_number().await?;

        //  Perform binary searches for start and end block in concurrent
        let (start_block_result, end_block_result) = join!(
            self.account_provider.binary_search_for_nonce(
                start_nonce,
                &sender,
                0,
                latest_block_number
            ),
            self.account_provider.binary_search_for_nonce(
                end_nonce + 1,
                &sender,
                0,
                latest_block_number
            )
        );

        let start_block = start_block_result?;
        let end_block = end_block_result?;

        // ideally offset should be 1, but somehow etherscan api is unstable for last and first element
        let offset = end_nonce - start_nonce + 2;

        let target_nonce_range: Vec<u64> = (start_nonce..=end_nonce)
            .step_by(incremental as usize)
            .collect();

        // call etherscan get_tx_hashes_from_etherscan
        let tx_result = self
            .tx_provider
            .get_tx_hashes_from_etherscan(
                start_block,
                end_block,
                offset,
                sender,
                ETHERSCAN_API_KEY.to_string(),
                target_nonce_range.clone(),
            )
            .await?;

        let mut txs_mpt_handler = TxsMptHandler::new(self.account_provider.url).unwrap();
        for tx in &tx_result {
            let target_blocknumber = tx.block_number.parse().unwrap();
            txs_mpt_handler
                .build_tx_tree_from_block(target_blocknumber)
                .await
                .unwrap();
            let tx_index = txs_mpt_handler
                .tx_hash_to_tx_index(B256::from_str(&tx.hash).unwrap())
                .unwrap();
            let proof = txs_mpt_handler
                .get_proof(tx_index)
                .unwrap()
                .into_iter()
                .map(|x| Bytes::from(x).to_string())
                .collect::<Vec<_>>();
            let consensus_tx = txs_mpt_handler.get_tx(tx_index).unwrap();
            let rlp = Bytes::from(consensus_tx.rlp_encode()).to_string();
            tx_with_proof.push((target_blocknumber, tx_index, rlp, proof));
        }

        Ok(tx_with_proof)
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

    // const SEPOLIA_TARGET_ADDRESS: &str = "0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4";

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

    // #[tokio::test]
    // async fn get_block_range_from_massive_nonce_range() {
    //     let provider = AbstractProvider::new(SEPOLIA_RPC_URL, 11155111);
    //     let block_range = provider
    //         .get_block_range_from_nonce_range(62988, 63987, 1, SEPOLIA_TARGET_ADDRESS.to_string())
    //         .await
    //         .unwrap();

    //     let block_range: Vec<u64> = block_range
    //         .iter()
    //         .map(|block| block.block_number.parse().unwrap())
    //         .collect::<Vec<_>>();
    //     assert_eq!(block_range.len(), 1000);
    //     assert_eq!(
    //         block_range,
    //         vec![
    //             5596072, 5596082, 5596094, 5596103, 5596113, 5596123, 5596132, 5596142, 5596153,
    //             5596162, 5596172, 5596184, 5596192, 5596202, 5596213, 5596223, 5596232, 5596242,
    //             5596252, 5596262, 5596272, 5596282, 5596297, 5596302, 5596312, 5596322, 5596333,
    //             5596343, 5596353, 5596363, 5596373, 5596383, 5596392, 5596405, 5596413, 5596422,
    //             5596432, 5596442, 5596452, 5596462, 5596472, 5596483, 5596492, 5596503, 5596514,
    //             5596523, 5596532, 5596544, 5596554, 5596562, 5596572, 5596582, 5596592, 5596605,
    //             5596612, 5596633, 5596634, 5596644, 5596654, 5596666, 5596676, 5596684, 5596694,
    //             5596704, 5596714, 5596725, 5596735, 5596744, 5596754, 5596764, 5596777, 5596784,
    //             5596794, 5596805, 5596815, 5596824, 5596835, 5596845, 5596855, 5596865, 5596876,
    //             5596884, 5596894, 5596904, 5596914, 5596924, 5596934, 5596947, 5596954, 5596964,
    //             5596974, 5596984, 5596995, 5597004, 5597014, 5597024, 5597035, 5597047, 5597054,
    //             5597064, 5597075, 5597086, 5597094, 5597104, 5597114, 5597124, 5597135, 5597144,
    //             5597156, 5597164, 5597174, 5597186, 5597194, 5597204, 5597214, 5597224, 5597237,
    //             5597246, 5597254, 5597264, 5597274, 5597284, 5597295, 5597308, 5597314, 5597325,
    //             5597337, 5597344, 5597354, 5597364, 5597374, 5597384, 5597394, 5597405, 5597415,
    //             5597424, 5597434, 5597444, 5597454, 5597464, 5597475, 5597484, 5597494, 5597504,
    //             5597515, 5597524, 5597536, 5597546, 5597554, 5597564, 5597574, 5597584, 5597594,
    //             5597604, 5597615, 5597624, 5597634, 5597644, 5597654, 5597664, 5597674, 5597684,
    //             5597695, 5597705, 5597714, 5597724, 5597734, 5597744, 5597754, 5597764, 5597776,
    //             5597788, 5597794, 5597804, 5597814, 5597824, 5597834, 5597845, 5597855, 5597864,
    //             5597875, 5597884, 5597894, 5597905, 5597914, 5597924, 5597934, 5597945, 5597955,
    //             5597965, 5597974, 5597984, 5597994, 5598005, 5598014, 5598024, 5598035, 5598044,
    //             5598054, 5598064, 5598074, 5598084, 5598094, 5598104, 5598114, 5598124, 5598134,
    //             5598144, 5598154, 5598166, 5598174, 5598184, 5598194, 5598204, 5598214, 5598224,
    //             5598234, 5598244, 5598254, 5598264, 5598274, 5598284, 5598294, 5598304, 5598314,
    //             5598324, 5598334, 5598344, 5598354, 5598364, 5598374, 5598384, 5598394, 5598404,
    //             5598414, 5598424, 5598434, 5598444, 5598454, 5598464, 5598474, 5598484, 5598494,
    //             5598504, 5598514, 5598525, 5598534, 5598545, 5598554, 5598565, 5598575, 5598584,
    //             5598594, 5598604, 5598614, 5598624, 5598634, 5598645, 5598654, 5598664, 5598674,
    //             5598684, 5598694, 5598704, 5598714, 5598724, 5598734, 5598744, 5598754, 5598764,
    //             5598774, 5598784, 5598794, 5598804, 5598814, 5598824, 5598836, 5598845, 5598854,
    //             5598864, 5598874, 5598884, 5598894, 5598904, 5598914, 5598925, 5598935, 5598944,
    //             5598955, 5598964, 5598976, 5598984, 5598994, 5599004, 5599015, 5599024, 5599035,
    //             5599045, 5599054, 5599064, 5599074, 5599085, 5599094, 5599104, 5599114, 5599124,
    //             5599134, 5599144, 5599155, 5599164, 5599175, 5599184, 5599194, 5599204, 5599215,
    //             5599224, 5599234, 5599244, 5599255, 5599265, 5599274, 5599284, 5599294, 5599306,
    //             5599314, 5599324, 5599334, 5599345, 5599354, 5599364, 5599374, 5599384, 5599394,
    //             5599404, 5599414, 5599424, 5599435, 5599446, 5599454, 5599464, 5599474, 5599484,
    //             5599494, 5599504, 5599514, 5599524, 5599534, 5599544, 5599554, 5599564, 5599574,
    //             5599585, 5599596, 5599604, 5599614, 5599626, 5599634, 5599644, 5599654, 5599664,
    //             5599674, 5599684, 5599694, 5599704, 5599714, 5599724, 5599735, 5599744, 5599754,
    //             5599764, 5599774, 5599784, 5599794, 5599804, 5599814, 5599824, 5599835, 5599844,
    //             5599854, 5599864, 5599874, 5599884, 5599894, 5599904, 5599914, 5599924, 5599934,
    //             5599944, 5599954, 5599964, 5599974, 5599984, 5599994, 5600006, 5600014, 5600028,
    //             5600034, 5600044, 5600054, 5600064, 5600074, 5600084, 5600094, 5600104, 5600114,
    //             5600124, 5600134, 5600144, 5600154, 5600164, 5600174, 5600184, 5600194, 5600204,
    //             5600214, 5600224, 5600234, 5600244, 5600254, 5600264, 5600274, 5600284, 5600294,
    //             5600304, 5600314, 5600325, 5600334, 5600344, 5600354, 5600364, 5600374, 5600384,
    //             5600394, 5600404, 5600414, 5600424, 5600434, 5600444, 5600454, 5600464, 5600474,
    //             5600484, 5600494, 5600504, 5600514, 5600524, 5600534, 5600544, 5600554, 5600564,
    //             5600575, 5600584, 5600594, 5600604, 5600614, 5600624, 5600634, 5600644, 5600654,
    //             5600665, 5600674, 5600684, 5600694, 5600704, 5600714, 5600724, 5600734, 5600744,
    //             5600754, 5600764, 5600774, 5600784, 5600794, 5600804, 5600814, 5600824, 5600834,
    //             5600844, 5600854, 5600864, 5600874, 5600884, 5600894, 5600904, 5600914, 5600924,
    //             5600934, 5600944, 5600954, 5600964, 5600974, 5600984, 5600994, 5601004, 5601014,
    //             5601024, 5601034, 5601044, 5601054, 5601066, 5601075, 5601084, 5601095, 5601104,
    //             5601114, 5601124, 5601134, 5601144, 5601154, 5601164, 5601174, 5601184, 5601194,
    //             5601204, 5601214, 5601224, 5601234, 5601244, 5601254, 5601264, 5601274, 5601284,
    //             5601294, 5601304, 5601314, 5601324, 5601334, 5601344, 5601354, 5601364, 5601374,
    //             5601384, 5601395, 5601404, 5601414, 5601424, 5601434, 5601444, 5601454, 5601464,
    //             5601474, 5601484, 5601494, 5601504, 5601514, 5601524, 5601534, 5601544, 5601554,
    //             5601564, 5601574, 5601584, 5601594, 5601604, 5601614, 5601624, 5601634, 5601644,
    //             5601654, 5601664, 5601674, 5601684, 5601694, 5601704, 5601714, 5601724, 5601734,
    //             5601744, 5601754, 5601764, 5601774, 5601784, 5601794, 5601804, 5601814, 5601824,
    //             5601834, 5601844, 5601854, 5601864, 5601874, 5601884, 5601894, 5601904, 5601914,
    //             5601924, 5601934, 5601944, 5601954, 5601964, 5601974, 5601984, 5601994, 5602004,
    //             5602014, 5602024, 5602034, 5602044, 5602054, 5602064, 5602074, 5602085, 5602096,
    //             5602106, 5602114, 5602124, 5602134, 5602144, 5602154, 5602164, 5602174, 5602184,
    //             5602194, 5602204, 5602214, 5602225, 5602234, 5602244, 5602254, 5602264, 5602275,
    //             5602284, 5602294, 5602304, 5602314, 5602324, 5602334, 5602344, 5602354, 5602364,
    //             5602374, 5602384, 5602394, 5602404, 5602414, 5602424, 5602434, 5602444, 5602457,
    //             5602464, 5602474, 5602484, 5602494, 5602504, 5602514, 5602524, 5602534, 5602544,
    //             5602554, 5602564, 5602574, 5602584, 5602594, 5602604, 5602614, 5602624, 5602635,
    //             5602644, 5602654, 5602664, 5602674, 5602684, 5602694, 5602704, 5602714, 5602724,
    //             5602734, 5602744, 5602754, 5602764, 5602774, 5602784, 5602794, 5602804, 5602814,
    //             5602824, 5602834, 5602844, 5602854, 5602864, 5602874, 5602884, 5602894, 5602904,
    //             5602914, 5602924, 5602934, 5602944, 5602954, 5602964, 5602974, 5602984, 5602994,
    //             5603004, 5603014, 5603024, 5603034, 5603044, 5603054, 5603064, 5603074, 5603084,
    //             5603094, 5603104, 5603114, 5603124, 5603134, 5603144, 5603154, 5603164, 5603174,
    //             5603184, 5603194, 5603204, 5603214, 5603224, 5603234, 5603244, 5603254, 5603264,
    //             5603274, 5603284, 5603294, 5603304, 5603314, 5603324, 5603334, 5603344, 5603354,
    //             5603364, 5603374, 5603385, 5603394, 5603404, 5603414, 5603426, 5603435, 5603444,
    //             5603455, 5603468, 5603474, 5603484, 5603494, 5603504, 5603514, 5603524, 5603534,
    //             5603544, 5603554, 5603564, 5603574, 5603584, 5603594, 5603604, 5603614, 5603624,
    //             5603634, 5603644, 5603654, 5603664, 5603675, 5603684, 5603694, 5603704, 5603714,
    //             5603724, 5603734, 5603744, 5603754, 5603764, 5603774, 5603785, 5603794, 5603804,
    //             5603814, 5603824, 5603834, 5603845, 5603854, 5603864, 5603874, 5603884, 5603894,
    //             5603904, 5603915, 5603924, 5603934, 5603945, 5603954, 5603964, 5603974, 5603984,
    //             5603994, 5604004, 5604014, 5604024, 5604034, 5604045, 5604054, 5604064, 5604074,
    //             5604084, 5604094, 5604104, 5604114, 5604124, 5604134, 5604144, 5604154, 5604164,
    //             5604175, 5604188, 5604194, 5604204, 5604214, 5604226, 5604234, 5604244, 5604254,
    //             5604265, 5604276, 5604284, 5604294, 5604304, 5604316, 5604324, 5604334, 5604344,
    //             5604354, 5604364, 5604376, 5604384, 5604396, 5604404, 5604414, 5604424, 5604435,
    //             5604444, 5604455, 5604464, 5604475, 5604484, 5604494, 5604505, 5604514, 5604524,
    //             5604534, 5604544, 5604554, 5604564, 5604575, 5604584, 5604594, 5604606, 5604614,
    //             5604627, 5604637, 5604644, 5604654, 5604665, 5604674, 5604684, 5604694, 5604705,
    //             5604714, 5604725, 5604734, 5604744, 5604754, 5604765, 5604774, 5604784, 5604796,
    //             5604804, 5604814, 5604828, 5604834, 5604845, 5604855, 5604864, 5604875, 5604890,
    //             5604894, 5604904, 5604914, 5604924, 5604934, 5604944, 5604954, 5604964, 5604974,
    //             5604986, 5604994, 5605004, 5605015, 5605024, 5605034, 5605044, 5605054, 5605064,
    //             5605075, 5605084, 5605094, 5605104, 5605114, 5605127, 5605134, 5605145, 5605154,
    //             5605164, 5605174, 5605185, 5605195, 5605204, 5605215, 5605224, 5605234, 5605244,
    //             5605254, 5605264, 5605274, 5605284, 5605294, 5605304, 5605314, 5605324, 5605334,
    //             5605344, 5605358, 5605364, 5605374, 5605384, 5605394, 5605404, 5605416, 5605425,
    //             5605438, 5605444, 5605454, 5605464, 5605474, 5605484, 5605495, 5605504, 5605514,
    //             5605524, 5605535, 5605544, 5605554, 5605565, 5605574, 5605586, 5605594, 5605604,
    //             5605614, 5605625, 5605634, 5605644, 5605654, 5605666, 5605677, 5605684, 5605694,
    //             5605705, 5605714, 5605724, 5605734, 5605744, 5605754, 5605764, 5605774, 5605784,
    //             5605796, 5605804, 5605814, 5605824, 5605834, 5605844, 5605854, 5605864, 5605875,
    //             5605884, 5605894, 5605905, 5605914, 5605925, 5605935, 5605944, 5605954, 5605964,
    //             5605975, 5605985, 5605996, 5606005, 5606014, 5606024, 5606036, 5606044, 5606054,
    //             5606066
    //         ]
    //     );
    // }

    // #[tokio::test]
    // async fn get_block_range_from_nonce_range() {
    //     let provider = AbstractProvider::new(SEPOLIA_RPC_URL, 11155111);
    //     let block_range = provider
    //         .get_block_range_from_nonce_range(63878, 63887, 1, SEPOLIA_TARGET_ADDRESS.to_string())
    //         .await
    //         .unwrap();
    //     let block_range: Vec<u64> = block_range
    //         .iter()
    //         .map(|block| block.block_number.parse().unwrap())
    //         .collect::<Vec<_>>();
    //     assert_eq!(block_range.len(), 10);
    //     assert_eq!(
    //         block_range,
    //         vec![
    //             5604974, 5604986, 5604994, 5605004, 5605015, 5605024, 5605034, 5605044, 5605054,
    //             5605064
    //         ]
    //     );
    // }

    const SEPOLIA_TARGET_ADDRESS_NON_CONSTANT: &str = "0x0a4De450feB156A2A51eD159b2fb99Da26E5F3A3";

    #[tokio::test]
    async fn get_block_range_from_nonce_range_non_constant() {
        let provider = AbstractProvider::new(SEPOLIA_RPC_URL, 11155111);
        let block_range = provider
            .get_tx_with_proof_from_nonce_range(
                520,
                524,
                1,
                SEPOLIA_TARGET_ADDRESS_NON_CONSTANT.to_string(),
            )
            .await
            .unwrap();

        assert_eq!(block_range.len(), 5);
        assert_eq!(block_range[0],(5530433, 97,"0x02f89483aa36a7820208843b9aca008441b8b4a88301024b948a6cc5e8368961bf5366b4684ce1c6ab9b8c2b2780a48ecc5fdc0000000000000000000000000000000000000000000000000000000000000003c001a0c2ddffedd382744f8d5d8ef294288fadd1f9bc7a4d352f0d2bb631d465d5183ba0665d4dd90e9523b913944b39aa61d6704a4f97db0567a3a8faf563ec6f8a7a69".to_string(), vec!["0xf90131a0ac1d10a49f4a25824a4b49096f2a9f82a29b20727e26b3e043a847aa6bf1730ba0ff2c099e28ccbb45137612d10daf13169a926ca30f65d322807a230f2d2ebbd2a02c96e5f9d7ea14e46aafbceb5f39a82a69556ea7ac05206857cc0ee52524a8b4a04967fab0cd77e99f89c705ff681ad56439e8f01b2918a6d89d5bd04c9c2eb0c2a0f6fbecf814db02bffd6e6a32aab227c5953eb037029acbde74c38c744d5e476fa024adcd7ba46f6a741c9130920c3bfdd0249d9a6f881ed7740662aaaed04f39c4a0c1455101baf2d24edd09a27ac82b1ba5ce58967442d058558140987bda1d37e6a032880e335daddb84399cb9af62ce4191eed895f15f33fcbbf63493b3b57983bba04d583d630b480433c164935183e3540c39b1108bde4c5b7ab65de95c99d7ada08080808080808080".to_string(), "0xf90211a0ad1cc7eb91fb0ff6aac3046fb15e2de21946f9422419bb34cd35512b38ede294a08b4ad4e4653d098c2aa617607e7569926ec74a5c9248bb01f4ec25836080cb36a09ba4dae637c01ba0e541ccfbd99b51038d33ec3a1ec0e9d07df92c4364630d40a098653c6965b3f22c305bd72f04406cc6f6fb2e9ed5b7e7cf64e9c7946b350549a00693c568d39b0d015164ecf76f48364fc1e985dfd295da32b15187a9d93082b4a01135fc8a3ecb8229da5e12454bedd2f3cedf629ec65ccaf09a773fd418321c7da0696c5ee3670d62d6ade62823d621a9116b9f02e84a895889372760c18ef40d2aa0d4330f6b5b082bf1fd66d332e8dc36662fffb1f2e801f9ea7563664a7622d8b8a0e4e628556596b88656989f83644d508b0a2400a9ab74b73ffcd923ffa6b0efa0a098aab6189c7bdb88738d7e3643d331ba9a4b55ffefb92a97071f3203e4b39412a05cbbe9d698b43279df5f174884679226ee11828a32744d58aefe06eba4a5a2b4a0d4fde2e14395ff212db4239a27ef46c7f3ddcc11fec4009ffe7fea86e913ac8ca0991c41a57d27b9acebb9b7556b69a7d65fdc746a583b6e7fb21696014eda3b8da041fad7f08078a5682cbf0d72b2838f84d833f766ee102d9b96dabf0cae565ca1a004b05eb016476620c1a087d48afc7fc1d6daf58609d8b3b1f2e424e64f17e891a0702ae55a972517f1151770842e28a4219f71695143144548b90a004b69ac8add80".to_string(), "0xf89a20b89702f89483aa36a7820208843b9aca008441b8b4a88301024b948a6cc5e8368961bf5366b4684ce1c6ab9b8c2b2780a48ecc5fdc0000000000000000000000000000000000000000000000000000000000000003c001a0c2ddffedd382744f8d5d8ef294288fadd1f9bc7a4d352f0d2bb631d465d5183ba0665d4dd90e9523b913944b39aa61d6704a4f97db0567a3a8faf563ec6f8a7a69".to_string()]))
    }
}
