use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Instant,
};

use alloy::primitives::BlockNumber;

use futures::future::join_all;
use jsonrpsee::{
    core::{client::ClientT, BoxError},
    http_client::{HttpClient, HttpClientBuilder},
    rpc_params,
};
use reqwest::Url;
use serde_json::json;
use starknet_types_core::felt::Felt;
use tokio::sync::{
    mpsc::{self, Sender},
    RwLock,
};
use tracing::{debug, error};

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
    let mut keys = Vec::new();
    if let Some(key) = storage_key {
        keys.push(format!("{}", key.to_hex_string()));
    }

    let request = json!({
        "jsonrpc": "2.0",
        "method": "pathfinder_getProof",
        "params": {
            "block_id": "latest",
            "contract_address": format!("{}", address.to_hex_string()),
            "keys": keys
        },
        "id": 0
    });

    println!("req:{}", request);

    let response: serde_json::Value = provider
        .request(
            "pathfinder_getProof",
            rpc_params![request["params"].clone()],
        )
        .await?;

    println!("res:{}", response);

    let get_proof_output: GetProofOutput = serde_json::from_value(response["result"].clone())?;
    Ok(get_proof_output)
}

async fn handle_proof_result(
    proof: Result<GetProofOutput, BoxError>,
    block_number: BlockNumber,
    blocks_map: Arc<RwLock<HashSet<BlockNumber>>>,
    rpc_sender: Sender<(BlockNumber, GetProofOutput)>,
) {
    match proof {
        Ok(proof) => {
            blocks_map.write().await.insert(block_number);
            rpc_sender.send((block_number, proof)).await.unwrap();
        }
        Err(e) => {
            error!("❗️❗️❗️ Error fetching proof: {:?}", e);
        }
    }
}
#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use super::*;
    use reqwest::Url;



    fn test_provider() -> RpcProvider {
        RpcProvider::new(Url::from_str(PATHFINDER_URL).unwrap(), 100)
    }

    #[tokio::test]
    async fn test_get_proof() {
        let provider = test_provider();
        let proof = provider
            .get_proofs(
                [156600].to_vec(),
                Felt::from_str(
                    "0x23371b227eaecd8e8920cd429d2cd0f3fee6abaacca08d3ab82a7cdd",
                )
                .unwrap(),
                Some(
                    Felt::from_str(
                        "0x1",
                    )
                    .unwrap(),
                ),
            )
            .await;

        println!("{:?}", proof);
    }
}
