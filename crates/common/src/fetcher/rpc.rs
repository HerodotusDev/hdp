use std::collections::HashMap;

use anyhow::{anyhow, bail, Result};
use reqwest::{header, Client};
use serde_json::{from_value, json, Value};

use crate::block::{
    account::AccountFromRpc,
    header::{BlockHeaderFromRpc, MMRFromIndexer, MMRMetaFromIndexer, MMRProofFromIndexer},
};

#[derive(Debug, Clone)]
pub struct RpcFetcher {
    client: Client,
    url: String,
}

impl RpcFetcher {
    pub fn new(rpc_url: String) -> Self {
        Self {
            client: Client::new(),
            url: rpc_url,
        }
    }
}

impl RpcFetcher {
    pub async fn get_block_by_number(&self, block_number: u64) -> Result<BlockHeaderFromRpc> {
        let rpc_request: Value = json!({
            "jsonrpc": "2.0",
            "method": "eth_getBlockByNumber",
            "params": [format!("0x{:x}", block_number), false],
            "id": 1,
        });

        let response = self
            .client
            .post(&self.url)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&rpc_request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request: {}", e))?;

        // Check if the response status is success
        if !response.status().is_success() {
            return Err(anyhow!(
                "RPC request `eth_getBlockByNumber` failed with status: {}",
                response.status()
            ));
        }

        // Parse the response body as JSON
        let rpc_response: Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse response: {}", e))?;
        let result = &rpc_response["result"];

        // Deserialize into EvmBlockHeaderFromRpc
        let block_header_from_rpc: BlockHeaderFromRpc = from_value(result.clone())?;

        Ok(block_header_from_rpc)
    }

    pub async fn get_proof(
        &self,
        block_number: u64,
        address: String,
        storage_keys: Option<Vec<String>>,
    ) -> Result<AccountFromRpc> {
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
            .post(&self.url)
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

        let account_from_rpc: AccountFromRpc = from_value(result.clone())?;

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

    pub async fn get_mmr_from_indexer(
        &self,
        block_numbers: &[u64],
    ) -> Result<(MMRMetaFromIndexer, HashMap<u64, MMRProofFromIndexer>)> {
        let blocks_query_params = block_numbers
            .iter()
            .map(|block_number| ("block_numbers".to_string(), block_number.to_string()))
            .collect::<Vec<(String, String)>>();

        let query_params = vec![
            ("deployed_on_chain".to_string(), "11155111".to_string()),
            ("accumulates_chain".to_string(), "11155111".to_string()),
            ("hashing_function".to_string(), "poseidon".to_string()),
            ("contract_type".to_string(), "AGGREGATOR".to_string()),
        ];

        let url = format!("{}/mmr-meta-and-proofs", &self.url);

        let response = self
            .client
            .get(url)
            .header(header::CONTENT_TYPE, "application/json")
            .query(&query_params)
            .query(&blocks_query_params)
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

        let mmr_from_indexer: MMRFromIndexer = from_value(rpc_response)?;

        // format into blocknumber -> mmr proof
        let mut mmr_from_indexer_map: HashMap<u64, MMRProofFromIndexer> = HashMap::new();
        for proof in &mmr_from_indexer.data[0].proofs {
            mmr_from_indexer_map.insert(proof.block_number, proof.clone());
        }

        Ok((mmr_from_indexer.data[0].meta.clone(), mmr_from_indexer_map))
    }
}
