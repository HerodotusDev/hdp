use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    vec,
};

use anyhow::{anyhow, bail, Result};
use futures::future::join_all;
use reqwest::{header, Client};
use serde_json::{from_value, json, Value};

use hdp_primitives::block::{
    account::{Account, AccountProofFromRpc},
    header::{MMRFromNewIndexer, MMRMetaFromNewIndexer, MMRProofFromNewIndexer},
};
use tokio::sync::{mpsc::Sender, RwLock};
use tracing::debug;

#[derive(Debug, Clone)]
pub struct FetchedAccountProof {
    pub block_number: u64,
    pub encoded_account: String,
    pub account_proof: Vec<String>,
}

/// Fetched storage and account proof and it's value
#[derive(Debug, Clone)]
pub struct FetchedStorageAccountProof {
    pub block_number: u64,
    pub encoded_account: String,
    pub account_proof: Vec<String>,
    pub storage_value: String,
    pub storage_proof: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct FetchedTransactionProof {
    pub block_number: u64,
    pub tx_index: u64,
    pub encoded_transaction: String,
    pub transaction_proof: Vec<String>,
    pub tx_type: u8,
}

#[derive(Debug, Clone)]
pub struct FetchedTransactionReceiptProof {
    pub block_number: u64,
    pub tx_index: u64,
    pub encoded_receipt: String,
    pub receipt_proof: Vec<String>,
    pub tx_type: u8,
}

pub struct HeaderProvider {
    client: Client,
    pub url: &'static str,
    chain_id: u64,
}

impl HeaderProvider {
    pub fn new(rpc_url: &'static str, chain_id: u64) -> Self {
        Self {
            client: Client::new(),
            url: rpc_url,
            chain_id,
        }
    }

    // TODO: result should not chunked
    pub async fn get_sequencial_headers_and_mmr_from_indexer(
        &self,
        from_block: u64,
        to_block: u64,
    ) -> Result<(MMRMetaFromNewIndexer, HashMap<u64, MMRProofFromNewIndexer>)> {
        let query_params = vec![
            ("deployed_on_chain".to_string(), self.chain_id.to_string()),
            ("accumulates_chain".to_string(), self.chain_id.to_string()),
            ("hashing_function".to_string(), "poseidon".to_string()),
            ("contract_type".to_string(), "AGGREGATOR".to_string()),
            (
                "from_block_number_inclusive".to_string(),
                from_block.to_string(),
            ),
            (
                "to_block_number_inclusive".to_string(),
                to_block.to_string(),
            ),
            ("is_meta_included".to_string(), "true".to_string()),
            ("is_whole_tree".to_string(), "true".to_string()),
            ("is_rlp_included".to_string(), "true".to_string()),
            ("is_pure_rlp".to_string(), "true".to_string()),
        ];

        let url = format!("{}/proofs", &self.url);

        let response = self
            .client
            .get(url)
            .header(header::CONTENT_TYPE, "application/json")
            .query(&query_params)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request: {}", e))?;

        // Check if the response status is success
        if !response.status().is_success() {
            bail!(
                "rs-indexer request failed with status: {}",
                response.status()
            );
        }

        // Parse the response body as JSON
        let rpc_response: Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse response: {}", e))?;

        let mmr_from_indexer: MMRFromNewIndexer = from_value(rpc_response)?;

        if mmr_from_indexer.data.is_empty() {
            bail!(
                "No MMR data found for block numbers: {} - {}",
                from_block,
                to_block
            );
        } else if mmr_from_indexer.data.len() > 1 {
            bail!(
                "More than one MMR data found for block numbers: {} - {}",
                from_block,
                to_block
            );
        } else {
            // As we are requesting for one tree, we expect only one tree to be returned
            // sort the proofs by block number
            // TODO: This sorting should be done in the indexer side
            let mut mmr_from_indexer_map: HashMap<u64, MMRProofFromNewIndexer> = HashMap::new();
            for proof in &mmr_from_indexer.data[0].proofs {
                mmr_from_indexer_map.insert(proof.block_number, proof.clone());
            }

            Ok((mmr_from_indexer.data[0].meta.clone(), mmr_from_indexer_map))
        }
    }
}

pub struct TrieProofProvider {
    client: Client,
    pub url: &'static str,
    chunk_size: u64,
}

impl TrieProofProvider {
    pub fn new(rpc_url: &'static str, chunk_size: u64) -> Self {
        Self {
            client: Client::new(),
            url: rpc_url,
            chunk_size,
        }
    }

    pub async fn get_account_proofs(
        &self,
        rpc_sender: Sender<FetchedAccountProof>,
        block_numbers: Vec<u64>,
        address: &str,
    ) {
        let url = self.url;
        let address = address.to_string();
        let chunk_size = self.chunk_size;

        debug!(
            "Fetching account proofs for {} chunk size: {}",
            address, chunk_size
        );

        tokio::spawn(async move {
            let mut try_count = 0;
            let blocks_map = Arc::new(RwLock::new(HashSet::<u64>::new()));

            while blocks_map.read().await.len() < block_numbers.len() {
                try_count += 1;
                if try_count > 50 {
                    panic!("❗️❗️❗️ Too many retries, failed to fetch all blocks")
                }
                let fetched_blocks_clone = blocks_map.read().await.clone();
                let blocks_to_fetch: Vec<u64> = block_numbers
                    .iter()
                    .filter(|block_number| !fetched_blocks_clone.contains(*block_number))
                    .take(chunk_size as usize)
                    .cloned()
                    .collect();

                let fetch_futures = blocks_to_fetch
                    .iter()
                    .map(|block_number| {
                        let fetched_blocks_clone = blocks_map.clone();
                        let rpc_sender = rpc_sender.clone();
                        let address = address.clone();
                        async move {
                            let account_from_rpc = TrieProofProvider::new(url, chunk_size)
                                .get_proof(*block_number, &address, None)
                                .await;
                            match account_from_rpc {
                                Ok(account_from_rpc) => {
                                    let mut blocks_identifier = fetched_blocks_clone.write().await;
                                    let account = Account::from(&account_from_rpc);
                                    let encoded_account = account.rlp_encode();
                                    let account_proof = account_from_rpc.account_proof;
                                    let mpt_proof = FetchedAccountProof {
                                        block_number: *block_number,
                                        encoded_account,
                                        account_proof,
                                    };
                                    rpc_sender.send(mpt_proof).await.unwrap();
                                    blocks_identifier.insert(*block_number);
                                }
                                Err(_) => {
                                    // println!(
                                    //     "Failed to fetch block number: {}, error: {}",
                                    //     block_number, e
                                    // );
                                }
                            }
                        }
                    })
                    .collect::<Vec<_>>();

                join_all(fetch_futures).await;
            }
        });
    }

    pub async fn get_storage_proofs(
        &self,
        rpc_sender: Sender<FetchedStorageAccountProof>,
        block_numbers: Vec<u64>,
        address: &str,
        slot: String,
    ) {
        let url = self.url;
        let address = address.to_string();
        let chunk_size = self.chunk_size;

        tokio::spawn(async move {
            let mut try_count = 0;
            let blocks_map = Arc::new(RwLock::new(HashSet::<u64>::new()));

            while blocks_map.read().await.len() < block_numbers.len() {
                try_count += 1;
                if try_count > 50 {
                    panic!("❗️❗️❗️ Too many retries, failed to fetch all blocks")
                }
                let fetched_blocks_clone = blocks_map.read().await.clone();
                let blocks_to_fetch: Vec<u64> = block_numbers
                    .iter()
                    .filter(|block_number| !fetched_blocks_clone.contains(*block_number))
                    .take(chunk_size as usize)
                    .cloned()
                    .collect();

                // println!("Fetching blocks: {:?}", blocks_to_fetch);

                let fetch_futures = blocks_to_fetch
                    .iter()
                    .map(|block_number| {
                        let fetched_blocks_clone = blocks_map.clone();
                        let rpc_sender = rpc_sender.clone();
                        let address = address.clone();
                        let slot = slot.clone();
                        async move {
                            let account_from_rpc = TrieProofProvider::new(url, chunk_size)
                                .get_proof(*block_number, &address, Some(vec![slot.clone()]))
                                .await;
                            match account_from_rpc {
                                Ok(account_from_rpc) => {
                                    let mut blocks_identifier = fetched_blocks_clone.write().await;
                                    let account = Account::from(&account_from_rpc);
                                    let encoded_account = account.rlp_encode();
                                    let storage = &account_from_rpc.storage_proof[0];
                                    let storage_value = storage.value.clone();
                                    let storage_proof =
                                        account_from_rpc.storage_proof[0].proof.clone();
                                    let account_proof = account_from_rpc.account_proof;
                                    let mpt_proof = FetchedStorageAccountProof {
                                        block_number: *block_number,
                                        encoded_account,
                                        account_proof,
                                        storage_value,
                                        storage_proof,
                                    };
                                    rpc_sender.send(mpt_proof).await.unwrap();
                                    blocks_identifier.insert(*block_number);
                                }
                                Err(_) => {
                                    // println!(
                                    //     "Failed to fetch block number: {}, error: {}",
                                    //     block_number, e
                                    // );
                                }
                            }
                        }
                    })
                    .collect::<Vec<_>>();

                join_all(fetch_futures).await;
            }
        });
    }

    async fn get_proof(
        &self,
        block_number: u64,
        address: &str,
        storage_keys: Option<Vec<String>>,
    ) -> Result<AccountProofFromRpc> {
        let storage_key_param = storage_keys.unwrap_or_default();

        let target_num = if block_number == u64::MAX {
            "latest".to_string()
        } else {
            format!("0x{:x}", block_number)
        };

        let rpc_request: Value = json!({
            "jsonrpc": "2.0",
            "method": "eth_getProof",
            "params": [
                address,
                storage_key_param,
                target_num,
            ],
            "id": 1,
        });

        let response = self
            .client
            .post(self.url)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&rpc_request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request: {}", e))?;

        // Check if the response status is success
        if !response.status().is_success() {
            bail!(
                "RPC request `eth_getProof` failed with status: {}",
                response.status()
            );
        }

        // Parse the response body as JSON
        let rpc_response: Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse response: {}", e))?;
        let result = &rpc_response["result"];

        let account_from_rpc: AccountProofFromRpc = from_value(result.clone())?;

        // Error handling for empty proof (no account found)
        if account_from_rpc.account_proof.is_empty() {
            bail!(
                "No account found for address {} in blocknumber {}",
                address,
                block_number
            );
        }

        // For now we only request for one storage key
        if !storage_key_param.is_empty() && account_from_rpc.storage_proof[0].proof.is_empty() {
            bail!(
                "No storage proof found for address {} in blocknumber {}",
                address,
                block_number
            );
        }

        Ok(account_from_rpc)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy_primitives::{FixedBytes, U256};
    use hdp_primitives::block::account::Account;

    use super::*;

    const HERODOTUS_RS_INDEXER_URL: &str = "https://rs-indexer.api.herodotus.cloud/accumulators";

    #[tokio::test]
    async fn test_get_sepolia_sequencial_headers_and_mmr_from_indexer() {
        let rpc_provider = HeaderProvider::new(HERODOTUS_RS_INDEXER_URL, 11155111);

        let block_header = rpc_provider
            .get_sequencial_headers_and_mmr_from_indexer(4952200, 4952229)
            .await
            .unwrap();

        let _ = &block_header.0;
        // assert_eq!(mmr_meta.mmr_id, 2);
        let length = block_header.1.len();
        assert_eq!(length, 30);
        let block_4952200 = block_header.1.get(&4952200).unwrap();
        assert_eq!(block_4952200.block_number, 4952200);

        let block_4952229 = block_header.1.get(&4952229).unwrap();
        assert_eq!(block_4952229.block_number, 4952229);
    }

    #[tokio::test]
    async fn test_get_mainnet_sequencial_headers_and_mmr_from_indexer() {
        let rpc_provider = HeaderProvider::new(HERODOTUS_RS_INDEXER_URL, 1);

        let block_header = rpc_provider
            .get_sequencial_headers_and_mmr_from_indexer(4952200, 4952229)
            .await
            .unwrap();

        let mmr_meta = &block_header.0;
        assert_eq!(mmr_meta.mmr_id, 6);
        let length = block_header.1.len();
        assert_eq!(length, 30);
        let block_4952200 = block_header.1.get(&4952200).unwrap();
        assert_eq!(block_4952200.block_number, 4952200);

        let block_4952229 = block_header.1.get(&4952229).unwrap();
        assert_eq!(block_4952229.block_number, 4952229);
    }

    // Non-paid personal alchemy endpoint
    const SEPOLIA_RPC_URL: &str =
        "https://eth-sepolia.g.alchemy.com/v2/a-w72ZvoUS0dfMD_LBPAuRzHOlQEhi_m";

    const SEPOLIA_TARGET_ADDRESS: &str = "0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4";

    #[tokio::test]
    async fn test_rpc_get_proof() {
        let rpc_provider = TrieProofProvider::new(SEPOLIA_RPC_URL, 40);

        let account_from_rpc = rpc_provider
            .get_proof(4952229, SEPOLIA_TARGET_ADDRESS, None)
            .await
            .unwrap();
        let account: Account = Account::from(&account_from_rpc);
        let expected_account = Account::new(
            6789,
            U256::from_str_radix("41694965332469803456", 10).unwrap(),
            FixedBytes::from_str(
                "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
            )
            .unwrap(),
            FixedBytes::from_str(
                "0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470",
            )
            .unwrap(),
        );
        assert_eq!(account, expected_account);
    }
}
