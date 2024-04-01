use std::collections::HashMap;

use anyhow::{anyhow, bail, Result};
use reqwest::{header, Client};
use serde_json::{from_value, json, Value};

use hdp_primitives::block::{
    account::AccountFromRpc,
    header::{
        BlockHeaderFromRpc, MMRFromNewIndexer, MMRMetaFromNewIndexer, MMRProofFromNewIndexer,
    },
};

#[derive(Debug, Clone)]
pub struct RpcProvider {
    client: Client,
    url: &'static str,
    chain_id: u64,
}

impl RpcProvider {
    pub fn new(rpc_url: &'static str, chain_id: u64) -> Self {
        Self {
            client: Client::new(),
            url: rpc_url,
            chain_id,
        }
    }
}

impl RpcProvider {
    pub async fn get_latest_block_number(&self) -> Result<u64> {
        let rpc_request: Value = json!({
            "jsonrpc": "2.0",
            "method": "eth_blockNumber",
            "params": [],
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
            return Err(anyhow!(
                "RPC request `eth_blockNumber` failed with status: {}",
                response.status()
            ));
        }

        // Parse the response body as JSON
        let rpc_response: Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse response: {}", e))?;
        let result = &rpc_response["result"];

        let block_number: String = from_value(result.clone())?;
        let block_number_u64 = u64::from_str_radix(&block_number[2..], 16).unwrap();

        Ok(block_number_u64)
    }
    pub async fn get_transaction_count(&self, address: &str, block_number: u64) -> Result<u64> {
        let rpc_request: Value = json!({
            "jsonrpc": "2.0",
            "method": "eth_getTransactionCount",
            "params": [address, format!("0x{:x}", block_number)],
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
            return Err(anyhow!(
                "RPC request `eth_getTransactionCount` failed with status: {}",
                response.status()
            ));
        }

        // Parse the response body as JSON
        let rpc_response: Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse response: {}", e))?;
        let result = &rpc_response["result"];

        let tx_count: String = from_value(result.clone())?;
        let tx_count_u64 = u64::from_str_radix(&tx_count[2..], 16).unwrap();

        Ok(tx_count_u64)
    }

    pub async fn get_block_by_number(&self, block_number: u64) -> Result<BlockHeaderFromRpc> {
        let rpc_request: Value = json!({
            "jsonrpc": "2.0",
            "method": "eth_getBlockByNumber",
            "params": [format!("0x{:x}", block_number), false],
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
        address: &str,
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

#[cfg(test)]
mod tests {
    use super::*;

    const HERODOTUS_RS_INDEXER_URL: &str = "https://rs-indexer.api.herodotus.cloud/accumulators";

    #[tokio::test]
    async fn test_get_sepolia_sequencial_headers_and_mmr_from_indexer() {
        let rpc_provider = RpcProvider::new(HERODOTUS_RS_INDEXER_URL, 11155111);

        let block_header = rpc_provider
            .get_sequencial_headers_and_mmr_from_indexer(4952200, 4952229)
            .await
            .unwrap();

        let mmr_meta = &block_header.0;
        assert_eq!(mmr_meta.mmr_id, 2);
        let length = block_header.1.len();
        assert_eq!(length, 30);
        let block_4952200 = block_header.1.get(&4952200).unwrap();
        assert_eq!(block_4952200.block_number, 4952200);

        let block_4952229 = block_header.1.get(&4952229).unwrap();
        assert_eq!(block_4952229.block_number, 4952229);
    }

    #[tokio::test]
    async fn test_get_mainnet_sequencial_headers_and_mmr_from_indexer() {
        let rpc_provider = RpcProvider::new(HERODOTUS_RS_INDEXER_URL, 1);

        let block_header = rpc_provider
            .get_sequencial_headers_and_mmr_from_indexer(4952200, 4952229)
            .await
            .unwrap();

        let mmr_meta = &block_header.0;
        assert_eq!(mmr_meta.mmr_id, 3);
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
    async fn test_get_transaction_count() {
        let rpc_provider = RpcProvider::new(SEPOLIA_RPC_URL, 1);

        let tx_count = rpc_provider
            .get_transaction_count(SEPOLIA_TARGET_ADDRESS, 4952200)
            .await
            .unwrap();

        assert_eq!(tx_count, 6786);

        let tx_count = rpc_provider
            .get_transaction_count(SEPOLIA_TARGET_ADDRESS, 4942101)
            .await
            .unwrap();

        assert_eq!(tx_count, 5776);
    }
}
