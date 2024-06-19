//! This module provides a `TrieValueProvider` struct that fetches trie values from an Ethereum node using JSON-RPC.

use alloy_primitives::B256;
use anyhow::{anyhow, Result};
use hdp_primitives::block::{account::AccountProofFromRpc, header::BlockHeaderFromRpc};
use reqwest::{header, Client};
use serde_json::{from_value, json, Value};
use std::str::FromStr;

use crate::key::{AccountProviderKey, HeaderProviderKey, StorageProviderKey};

pub struct TrieValueProvider {
    client: Client,
    pub url: &'static str,
}

impl TrieValueProvider {
    pub fn new(rpc_url: &'static str) -> Self {
        Self {
            client: Client::new(),
            url: rpc_url,
        }
    }

    pub async fn get_blocks_by_keys(
        &self,
        provider_keys: &[HeaderProviderKey],
    ) -> Result<Vec<BlockHeaderFromRpc>> {
        let mut blocks = Vec::new();
        for provider_key in provider_keys {
            let block = self.get_block_by_number(provider_key).await?;
            blocks.push(block);
        }
        Ok(blocks)
    }

    async fn get_block_by_number(
        &self,
        provider_key: &HeaderProviderKey,
    ) -> Result<BlockHeaderFromRpc> {
        let rpc_request: Value = json!({
            "jsonrpc": "2.0",
            "method": "eth_getBlockByNumber",
            "params": [format!("0x{:x}", provider_key.block_number), false],
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

    pub async fn get_accounts_by_keys(
        &self,
        provider_keys: &[AccountProviderKey],
    ) -> Result<Vec<AccountProofFromRpc>> {
        let mut accounts = Vec::new();
        for provider_key in provider_keys {
            let account = self.get_account(provider_key).await?;
            accounts.push(account);
        }
        Ok(accounts)
    }

    async fn get_account(&self, provider_key: &AccountProviderKey) -> Result<AccountProofFromRpc> {
        let rpc_request: Value = json!({
            "jsonrpc": "2.0",
            "method": "eth_getProof",
            "params": [provider_key.address, Vec::<String>::new(), format!("0x{:x}", provider_key.block_number)],
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
                "RPC request `eth_getAccount` failed with status: {}",
                response.status()
            ));
        }

        // Parse the response body as JSON
        let rpc_response: Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse response: {}", e))?;
        let result = &rpc_response["result"];

        // Deserialize into AccountFromRpc
        let account_from_rpc: AccountProofFromRpc = from_value(result.clone())?;

        Ok(account_from_rpc)
    }

    pub async fn get_storages_by_keys(
        &self,
        provider_keys: &[StorageProviderKey],
    ) -> Result<Vec<B256>> {
        let mut storages = Vec::new();
        for provider_key in provider_keys {
            let storage = self.get_storage_at(provider_key).await?;
            storages.push(storage);
        }
        Ok(storages)
    }

    pub async fn get_storage_at(&self, provider_key: &StorageProviderKey) -> Result<B256> {
        let rpc_request: Value = json!({
            "jsonrpc": "2.0",
            "method": "eth_getStorageAt",
            "params": [provider_key.address, provider_key.key, format!("0x{:x}", provider_key.block_number)],
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
                "RPC request `eth_getStorageAt` failed with status: {}",
                response.status()
            ));
        }

        // Parse the response body as JSON
        let rpc_response: Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse response: {}", e))?;
        let result = &rpc_response["result"];

        let storage_value: String = from_value(result.clone())?;

        Ok(B256::from_str(&storage_value)?)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy_primitives::{address, Address, B256};
    use hdp_primitives::block::header::Header;

    use crate::{
        evm::rpc::values_provider::TrieValueProvider,
        key::{AccountProviderKey, HeaderProviderKey, StorageProviderKey},
    };

    // Non-paid personal alchemy endpoint
    const SEPOLIA_RPC_URL: &str =
        "https://eth-sepolia.g.alchemy.com/v2/a-w72ZvoUS0dfMD_LBPAuRzHOlQEhi_m";

    #[tokio::test]
    async fn test_get_block_by_number() {
        let rpc_provider = TrieValueProvider::new(SEPOLIA_RPC_URL);

        let provider_key = HeaderProviderKey::new(1155511, 0);
        let block = rpc_provider
            .get_block_by_number(&provider_key)
            .await
            .unwrap();
        let block_header = Header::from(&block);
        assert_eq!(block.get_block_hash(), block_header.get_block_hash());

        let provider_key = HeaderProviderKey::new(1155511, 5521772);
        let block = rpc_provider
            .get_block_by_number(&provider_key)
            .await
            .unwrap();
        let block_header = Header::from(&block);
        assert_eq!(block.get_block_hash(), block_header.get_block_hash());

        let provider_key = HeaderProviderKey::new(1155511, 421772);
        let block = rpc_provider
            .get_block_by_number(&provider_key)
            .await
            .unwrap();
        let block_header = Header::from(&block);
        assert_eq!(block.get_block_hash(), block_header.get_block_hash())
    }

    #[tokio::test]
    async fn test_get_account() {
        let rpc_provider = TrieValueProvider::new(SEPOLIA_RPC_URL);
        let target_address = address!("0a4De450feB156A2A51eD159b2fb99Da26E5F3A3");
        let provider_key = AccountProviderKey::new(1155511, 6127485, target_address);
        let account = rpc_provider.get_account(&provider_key).await.unwrap();
        let retrieved_address = Address::from_str(&account.address).unwrap();
        assert_eq!(retrieved_address, target_address);
    }

    #[tokio::test]
    async fn test_get_storage_at() {
        let rpc_provider = TrieValueProvider::new(SEPOLIA_RPC_URL);
        let target_address = address!("75CeC1db9dCeb703200EAa6595f66885C962B920");
        let target_slot = B256::ZERO;
        let provider_key = StorageProviderKey::new(1155511, 6127485, target_address, target_slot);
        let storage_value = rpc_provider.get_storage_at(&provider_key).await.unwrap();
        assert_eq!(
            storage_value,
            B256::from_str("0x00000000000000000000000041ad2bc63a2059f9b623533d87fe99887d794847")
                .unwrap()
        );
    }
}
