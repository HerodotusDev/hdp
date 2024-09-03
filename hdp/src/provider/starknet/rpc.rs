use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Instant,
};

use alloy::primitives::BlockNumber;

use futures::future::join_all;
use jsonrpsee::{
    core::{client::ClientT, BoxError, ClientError},
    http_client::{HttpClient, HttpClientBuilder},
    rpc_params,
};
use reqwest::{Client, Url};
use starknet_types_core::felt::Felt;
use tokio::sync::{
    mpsc::{self, Sender},
    RwLock,
};
use tracing::debug;

use crate::provider::error::RpcProviderError;

use super::types::GetProofOutput;

/// !Note: have to use pathfinder node as we need `pathfinder_getProof`
pub struct RpcProvider {
    client: HttpClient,
    chunk_size: u64,
}

impl RpcProvider {
    pub fn new(rpc_url: Url, chunk_size: u64) -> Self {
        let client = HttpClientBuilder::default().build(rpc_url).unwrap();
        Self { client, chunk_size }
    }

    pub async fn get_account_proofs(&self, blocks: Vec<BlockNumber>, address: Felt) {}

    async fn get_proofs(
        &self,
        blocks: Vec<BlockNumber>,
        address: Felt,
        storage_key: Option<Felt>,
    ) -> Result<HashMap<BlockNumber, GetProofOutput>, RpcProviderError> {
        let start_fetch = Instant::now();
        let (rpc_sender, mut rx) = mpsc::channel::<(BlockNumber, GetProofOutput)>(32);
        self.spawn_proof_fetcher(rpc_sender, blocks, address, storage_key);

        let mut fetched_proofs = HashMap::new();
        while let Some((block_number, proof)) = rx.recv().await {
            fetched_proofs.insert(block_number, proof);
        }
        let duration = start_fetch.elapsed();
        debug!("time taken (Fetch): {:?}", duration);

        Ok(fetched_proofs)
    }

    fn spawn_proof_fetcher(
        &self,
        rpc_sender: Sender<(BlockNumber, GetProofOutput)>,
        blocks: Vec<BlockNumber>,
        address: Felt,
        storage_key: Option<Felt>,
    ) {
        let chunk_size = self.chunk_size;
        let provider_clone = self.client.clone();
        let target_blocks_length = blocks.len();

        debug!(
            "fetching proofs for {}, with chunk size: {}",
            address, chunk_size
        );

        tokio::spawn(async move {
            let mut try_count = 0;
            let blocks_map = Arc::new(RwLock::new(HashSet::<BlockNumber>::new()));

            while blocks_map.read().await.len() < target_blocks_length {
                try_count += 1;
                if try_count > 50 {
                    panic!("❗️❗️❗️ Too many retries, failed to fetch all blocks")
                }
                let fetched_blocks_clone = blocks_map.read().await.clone();

                let blocks_to_fetch: Vec<BlockNumber> = blocks
                    .iter()
                    .filter(|block_number| !fetched_blocks_clone.contains(block_number))
                    .take(chunk_size as usize)
                    .cloned()
                    .collect();

                let fetch_futures = blocks_to_fetch
                    .into_iter()
                    .map(|block_number| {
                        let fetched_blocks_clone = blocks_map.clone();
                        let rpc_sender = rpc_sender.clone();
                        let provider_clone = provider_clone.clone();
                        async move {
                            let proof = pathfinder_get_proof(
                                &provider_clone,
                                address,
                                block_number,
                                storage_key,
                            )
                            .await;
                            handle_proof_result(
                                proof,
                                block_number,
                                fetched_blocks_clone,
                                rpc_sender,
                            )
                            .await;
                        }
                    })
                    .collect::<Vec<_>>();

                join_all(fetch_futures).await;
            }
        });
    }
}

/// Fetches proof (account or storage) for a given block number
async fn pathfinder_get_proof(
    provider: &HttpClient,
    address: Felt,
    block_number: BlockNumber,
    storage_key: Option<Felt>,
) -> Result<GetProofOutput, BoxError> {
    match storage_key {
        Some(key) => {
            let params = rpc_params!["param1", "param2"];
            let response: String = provider.request("method_name", params).await?;
            let get_proof_output: GetProofOutput = serde_json::from_str(&response)?;
            Ok(get_proof_output)
        }
        None => {
            let params = rpc_params!["param1", "param2"];
            let response: String = provider.request("method_name", params).await?;
            let get_proof_output: GetProofOutput = serde_json::from_str(&response)?;
            Ok(get_proof_output)
        }
    }
}

async fn handle_proof_result(
    proof: Result<GetProofOutput, BoxError>,
    block_number: BlockNumber,
    blocks_map: Arc<RwLock<HashSet<BlockNumber>>>,
    rpc_sender: Sender<(BlockNumber, GetProofOutput)>,
) {
}
