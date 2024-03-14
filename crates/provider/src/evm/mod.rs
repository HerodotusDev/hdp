use anyhow::{bail, Result};
use core::panic;
use futures::future::join_all;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Instant,
};
use tokio::sync::RwLock;
use tracing::{error, info};

use hdp_primitives::{
    block::{account::Account, header::BlockHeader},
    format::MMRMeta,
};

use self::{
    memory::{MemoryFetcher, RlpEncodedValue, StoredHeader, StoredHeaders},
    rpc::RpcFetcher,
};

pub(crate) mod memory;
pub(crate) mod rpc;

/// [`AbstractFetcher`] abstracts the fetching of data from the RPC and memory.
///  It uses a [`MemoryFetcher`] and a [`RpcFetcher`] to fetch data.
///
/// TODO: Optimization idea, Lock only rpc fetcher and keep the memory fetcher unlocked
/// but handle requests so that it would not make duplicate requests
pub struct AbstractFetcher {
    /// [`MemoryFetcher`] is used to fetch data from memory.
    memory: MemoryFetcher,
    /// [`RpcFetcher`] is used to fetch data from the RPC.
    rpc: RpcFetcher,
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
    ) -> Result<(StoredHeaders, MMRMeta)> {
        //? A map of block numbers to a boolean indicating whether the block was fetched.
        let mut blocks_map: HashMap<u64, (bool, StoredHeader)> = HashMap::new();

        let mut relevant_mmr: HashSet<MMRMeta> = HashSet::new();

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
                info!("Successfully fetched MMR data from indexer");
                let duration = start_fetch.elapsed();
                info!("Time taken (fetch from Indexer): {:?}", duration);
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

                        relevant_mmr.insert(MMRMeta {
                            id: mmr_meta.mmr_id,
                            root: mmr_meta.mmr_root.clone(),
                            size: mmr_meta.mmr_size,
                            peaks: mmr_meta.mmr_peaks.clone(),
                        });

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
                info!("Time taken (during from Indexer): {:?}", duration);
                error!(
                    "Something went wrong while fetching MMR data from indexer: {}",
                    e
                );
                return Err(e);
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

        // TODO: in v1 allowed to handle all the blocks in datalake are exist in 1 MMR
        let mmr_meta_result = match relevant_mmr.len() {
            0 => None,
            1 => relevant_mmr.iter().next().cloned(),
            _ => relevant_mmr.iter().next().cloned(),
        };

        match mmr_meta_result {
            Some(mmr_meta) => Ok((stored_headers, mmr_meta)),
            None => bail!("No MMR metadata found"),
        }
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
            let account = self.memory.get_account(*block_number, address.clone());
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
                    let rpc_clone = self.rpc.clone(); // Assume self.rpc can be cloned or is wrapped in Arc
                    let address_clone = address.clone();
                    async move {
                        let should_fetch = {
                            let read_guard = blocks_map_clone.read().await;
                            !read_guard.contains_key(&block_number)
                        };
                        if should_fetch {
                            match rpc_clone.get_proof(block_number, address_clone, None).await {
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

    pub async fn get_storage_value_with_proof(
        &mut self,
        block_number: u64,
        account: String,
        slot: String,
    ) -> Result<(String, Vec<String>)> {
        match self
            .memory
            .get_storage(block_number, account.clone(), slot.clone())
        {
            Some(storage) => Ok(storage),
            None => {
                let account_rpc = self
                    .rpc
                    .get_proof(block_number, account.clone(), Some(vec![slot.clone()]))
                    .await?;
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
                Ok((storage_value, storage_proof))
            }
        }
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
            let account = self.memory.get_account(*block_number, address.clone());
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
                    let rpc_clone = self.rpc.clone(); // Assume self.rpc can be cloned or is wrapped in Arc
                    let address_clone = address.clone();
                    let slot_clone = slot.clone();
                    async move {
                        let should_fetch = {
                            let read_guard = blocks_map_clone.read().await;
                            !read_guard.contains_key(&block_number)
                        };
                        if should_fetch {
                            match rpc_clone
                                .get_proof(block_number, address_clone, Some(vec![slot_clone]))
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{hex, keccak256, FixedBytes, U256};
    use std::str::FromStr;

    use hdp_primitives::block::{account::Account, header::BlockHeader};

    fn rlp_string_to_block_hash(rlp_string: &str) -> String {
        keccak256(hex::decode(rlp_string).unwrap()).to_string()
    }

    const GOERLI_RPC_URL: &str =
        "https://eth-goerli.g.alchemy.com/v2/OcJWF4RZDjyeCWGSmWChIlMEV28LtA5c";

    #[tokio::test]
    async fn test_rpc_get_block_by_number() {
        let fetcher = RpcFetcher::new(GOERLI_RPC_URL.into());

        let block = fetcher.get_block_by_number(0).await.unwrap();
        let block_header = BlockHeader::from(&block);
        assert_eq!(block.get_block_hash(), block_header.get_block_hash());

        let block = fetcher.get_block_by_number(10487680).await.unwrap();
        let block_header = BlockHeader::from(&block);
        assert_eq!(block.get_block_hash(), block_header.get_block_hash());

        let block = fetcher.get_block_by_number(487680).await.unwrap();
        let block_header = BlockHeader::from(&block);
        assert_eq!(block.get_block_hash(), block_header.get_block_hash());
    }

    #[tokio::test]
    async fn test_rpc_get_proof() {
        let fetcher = RpcFetcher::new(GOERLI_RPC_URL.into());
        let target_address = "0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6".to_string();
        let account_from_rpc = fetcher
            .get_proof(10399990, target_address.clone(), None)
            .await
            .unwrap();
        let account: Account = Account::from(&account_from_rpc);
        let expected_account = Account::new(
            1,
            U256::from(0),
            FixedBytes::from_str(
                "0x480489b48e337887827fd9584f40dc1f51016e49df77ec789d4ee9bcc87bb0ff",
            )
            .unwrap(),
            FixedBytes::from_str(
                "0xcd4f25236fff0ccac15e82bf4581beb08e95e1b5ba89de6031c75893cd91245c",
            )
            .unwrap(),
        );
        assert_eq!(account, expected_account);
    }

    #[tokio::test]
    async fn test_fetcher_get_rlp_header() {
        let mut abstract_fetcher = AbstractFetcher::new(GOERLI_RPC_URL.into());
        let rlp_header = abstract_fetcher.get_rlp_header(0).await;
        let block_hash = rlp_string_to_block_hash(&rlp_header);
        assert_eq!(
            block_hash,
            "0xbf7e331f7f7c1dd2e05159666b3bf8bc7a8a3a9eb1d518969eab529dd9b88c1a"
        );
        let rlp_header = abstract_fetcher.get_rlp_header(10399990).await;
        let block_hash = rlp_string_to_block_hash(&rlp_header);
        assert_eq!(
            block_hash,
            "0x2ef5bd5264f472d821fb950241aa2bbe83f885fea086b4f58fccb9c9b948adcf"
        );
        let rlp_header = abstract_fetcher.get_rlp_header(487680).await;
        let block_hash = rlp_string_to_block_hash(&rlp_header);
        assert_eq!(
            block_hash,
            "0x9372b3057affe70c15a3a62dbdcb188677bdc8a403bc097acc22995544b27ba7"
        );
    }

    #[tokio::test]
    async fn test_fetcher_get_rlp_account() {
        let mut abstract_fetcher = AbstractFetcher::new(GOERLI_RPC_URL.into());
        let rlp_account = abstract_fetcher
            .get_account_with_proof(0, "0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6".to_string())
            .await;
        assert_eq!(rlp_account.0, "f8448080a00000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000");
        let rlp_account = abstract_fetcher
            .get_account_with_proof(
                10399990,
                "0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6".to_string(),
            )
            .await;
        assert_eq!(rlp_account.0, "f8440180a0480489b48e337887827fd9584f40dc1f51016e49df77ec789d4ee9bcc87bb0ffa0cd4f25236fff0ccac15e82bf4581beb08e95e1b5ba89de6031c75893cd91245c");
    }

    const SEPOLIA_RPC_URL: &str =
        "https://eth-sepolia.g.alchemy.com/v2/a-w72ZvoUS0dfMD_LBPAuRzHOlQEhi_m";

    #[tokio::test]
    async fn test_fetcher_get_non_exist_storage_value() {
        let mut abstract_fetcher = AbstractFetcher::new(SEPOLIA_RPC_URL.into());
        let storage_value = abstract_fetcher
            .get_storage_value_with_proof(
                0,
                "0x75CeC1db9dCeb703200EAa6595f66885C962B920".to_string(),
                "0x0000000000000000000000000000000000000000000000000000000000000002".to_string(),
            )
            .await;

        assert!(storage_value.is_err());

        let storage_value = abstract_fetcher
            .get_storage_value_with_proof(
                20,
                "0x75CeC1db9dCeb703200EAa6595f66885C962B920".to_string(),
                "0x0000000000000000000000000000000000000000000000000000000000000002".to_string(),
            )
            .await;
        assert!(storage_value.is_err());

        // Actually the storage value is not 0x0 for later block, in the case, proof is not empty
        let storage_value = abstract_fetcher
            .get_storage_value_with_proof(
                5382810,
                "0x75CeC1db9dCeb703200EAa6595f66885C962B920".to_string(),
                "0x0000000000000000000000000000000000000000000000000000000000000002".to_string(),
            )
            .await;
        assert!(storage_value.is_ok());
        let storage_value = storage_value.unwrap();
        assert_eq!(storage_value.0, "0x9184e72a000");
        assert_eq!(storage_value.1, vec!["0xf8918080a0b7a7c859e6ddbad6c18adb60b9f48842e652021b4f8b875894b8b879568629f880a0e7f9c6d331c7d110c992550a7baa3e051adc1e26a53d928dbd517a313d221863808080808080a0e40cf9c20b1e8e4aaf3201dd3cb84ab06d2bac34e8dc3e918626e5c44c4f0707808080a0c01a2f302bfc71151daac60eeb4c1b73470845d4fe219e71644752abaafb02ab80", "0xe9a0305787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5ace878609184e72a000"]);

        // Even actually storage value is 0x0, but the proof is not empty
        let storage_value = abstract_fetcher
            .get_storage_value_with_proof(
                5382769,
                "0x75CeC1db9dCeb703200EAa6595f66885C962B920".to_string(),
                "0x0000000000000000000000000000000000000000000000000000000000000002".to_string(),
            )
            .await;
        assert!(storage_value.is_ok());
        let storage_value = storage_value.unwrap();
        assert_eq!(storage_value.0, "0x0");
        assert_eq!(storage_value.1, vec!["0xf838a120290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563959441ad2bc63a2059f9b623533d87fe99887d794847"]);
    }
}
