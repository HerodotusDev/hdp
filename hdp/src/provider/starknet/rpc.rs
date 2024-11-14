use alloy::primitives::BlockNumber;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::Instant,
};

use futures::future::join_all;
use reqwest::{Client, Url};
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
    client: reqwest::Client,
    url: Url,
    chunk_size: u64,
}

impl RpcProvider {
    pub fn new(rpc_url: Url, chunk_size: u64) -> Self {
        Self {
            client: Client::new(),
            url: rpc_url,
            chunk_size,
        }
    }

    /// Get storage with proof in given vector of blocks and slot
    pub async fn get_storage_proofs(
        &self,
        blocks: Vec<BlockNumber>,
        address: Felt,
        storage_keys: Vec<Felt>,
    ) -> Result<HashMap<BlockNumber, GetProofOutput>, RpcProviderError> {
        let blocks_with_storage_keys = blocks
            .into_iter()
            .map(|block_number| (block_number, storage_keys.clone()))
            .collect();
        self.get_proofs(blocks_with_storage_keys, address).await
    }

    pub async fn get_proofs(
        &self,
        blocks_with_storage_keys: Vec<(BlockNumber, Vec<Felt>)>,
        address: Felt,
    ) -> Result<HashMap<BlockNumber, GetProofOutput>, RpcProviderError> {
        let start_fetch = Instant::now();
        let (rpc_sender, mut rx) = mpsc::channel::<(BlockNumber, GetProofOutput)>(32);
        self.spawn_proof_fetcher(rpc_sender, blocks_with_storage_keys, address);

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
        blocks_with_storage_keys: Vec<(BlockNumber, Vec<Felt>)>,
        address: Felt,
    ) {
        let chunk_size = self.chunk_size;
        let provider_clone = self.client.clone();
        let target_blocks_length = blocks_with_storage_keys.len();
        let url = self.url.clone();

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

                let blocks_and_keys_to_fetch: Vec<(BlockNumber, Vec<Felt>)> =
                    blocks_with_storage_keys
                        .iter()
                        .filter(|(block_number, _)| !fetched_blocks_clone.contains(block_number))
                        .take(chunk_size as usize)
                        .cloned()
                        .collect();

                let fetch_futures = blocks_and_keys_to_fetch
                    .into_iter()
                    .map(|(block_number, storage_keys)| {
                        let fetched_blocks_clone = blocks_map.clone();
                        let rpc_sender = rpc_sender.clone();
                        let provider_clone = provider_clone.clone();
                        let url = url.clone();
                        async move {
                            let proof = pathfinder_get_proof(
                                &provider_clone,
                                url,
                                address,
                                block_number,
                                storage_keys,
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
    provider: &reqwest::Client,
    url: Url,
    address: Felt,
    block_number: BlockNumber,
    storage_keys: Vec<Felt>,
) -> Result<GetProofOutput, RpcProviderError> {
    let keys: Vec<String> = storage_keys
        .into_iter()
        .map(|k| k.to_hex_string())
        .collect();

    let request = json!({
        "jsonrpc": "2.0",
        "id": "0",
        "method": "pathfinder_getProof",
        "params": {
            "block_id": {"block_number": block_number},
            "contract_address": format!("{}", address.to_hex_string()),
            "keys": keys
        }
    });

    let response = provider.post(url).json(&request).send().await?;
    let response_json =
        serde_json::from_str::<serde_json::Value>(&response.text().await?)?["result"].clone();
    let get_proof_output: GetProofOutput = serde_json::from_value(response_json)?;
    Ok(get_proof_output)
}

async fn handle_proof_result(
    proof: Result<GetProofOutput, RpcProviderError>,
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

    const PATHFINDER_URL: &str = "https://pathfinder.sepolia.iosis.tech/";

    fn test_provider() -> RpcProvider {
        RpcProvider::new(Url::from_str(PATHFINDER_URL).unwrap(), 100)
    }

    #[tokio::test]
    async fn test_get_100_range_storage_with_proof() {
        let target_block_start = 208383;
        let target_block_end = 208483;
        let target_block_range = (target_block_start..=target_block_end).collect::<Vec<u64>>();
        let provider = test_provider();
        let proof = provider
            .get_storage_proofs(
                target_block_range.clone(),
                Felt::from_str(
                    "0x017E2D0662675DD83B4B58A0A659EAFA131FDD01FA6DABD5002D8815DD2D17A5",
                )
                .unwrap(),
                vec![Felt::from_str(
                    "0x032ce6490b615c86e31587e14d6140e5a46231d9b8bf870fd708d71140c3ed2f",
                )
                .unwrap()],
            )
            .await
            .unwrap();

        assert_eq!(proof.len(), target_block_range.len());
        let output = proof.get(&target_block_start).unwrap();

        assert_eq!(
            output.state_commitment.unwrap(),
            Felt::from_str("0x16ba8b273b95235c11e0ad8c4238a510282495280df5abe7cbfb3c53e2d9c2d")
                .unwrap()
        );

        assert_eq!(output.contract_proof.len(), 19);

        assert_eq!(
            output.class_commitment.unwrap(),
            Felt::from_str("0x20cce483edc6fbb1290469dfacd96656414689fe82cf60bcc73cda7a1a8a90f")
                .unwrap()
        );

        assert_eq!(
            output.contract_data.clone().unwrap().storage_proofs[0].len(),
            4
        );
    }
}
