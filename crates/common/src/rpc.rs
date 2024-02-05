use anyhow::{anyhow, Result};
use reqwest::{header, Client};
use reth_primitives::Header;
use serde_json::{from_value, json, Value};

use crate::block::header::BlockHeaderFromRpc;

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
    pub async fn get_block_by_number(&self, block_number: u64) -> Result<Header> {
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
                "RPC request failed with status: {}",
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

        let block_header: Header = Header::from(&block_header_from_rpc);

        Ok(block_header)
    }
}
