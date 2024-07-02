use super::provider::EvmProvider;
use crate::evm::provider::ProviderError;
use crate::key::{
    AccountMemorizerKey, FetchKeyEnvelope, HeaderMemorizerKey, StorageMemorizerKey, TxMemorizerKey,
    TxReceiptMemorizerKey,
};
use alloy::primitives::{Address, BlockNumber, Bytes, TxIndex, B256};
use alloy::transports::{RpcError, TransportErrorKind};
use eth_trie_proofs::tx_receipt_trie::TxReceiptsMptHandler;
use eth_trie_proofs::tx_trie::TxsMptHandler;
use eth_trie_proofs::EthTrieError;
use hdp_primitives::processed_types::account::ProcessedAccount;
use hdp_primitives::processed_types::block_proofs::ProcessedBlockProofs;
use hdp_primitives::processed_types::header::ProcessedHeader;
use hdp_primitives::processed_types::mmr::MMRMeta;
use hdp_primitives::processed_types::mpt::ProcessedMPTProof;
use hdp_primitives::processed_types::receipt::ProcessedReceipt;
use hdp_primitives::processed_types::storage::ProcessedStorage;
use hdp_primitives::processed_types::transaction::ProcessedTransaction;
use std::collections::{HashMap, HashSet};
use std::time::Instant;
use tracing::info;

impl EvmProvider {
    /// This is the public entry point of provider.  
    pub async fn fetch_proofs_from_keys(
        &self,
        fetch_keys: Vec<FetchKeyEnvelope>,
    ) -> Result<ProcessedBlockProofs, ProviderError> {
        let mut target_keys_for_header = HashSet::new();
        let mut target_keys_for_account = HashSet::new();
        let mut target_keys_for_storage = HashSet::new();
        let mut target_keys_for_tx = HashSet::new();
        let mut target_keys_for_tx_receipt = HashSet::new();
        for key in fetch_keys {
            match key {
                FetchKeyEnvelope::Header(header_key) => {
                    target_keys_for_header.insert(header_key);
                }
                FetchKeyEnvelope::Account(account_key) => {
                    target_keys_for_header.insert(HeaderMemorizerKey::new(
                        account_key.chain_id,
                        account_key.block_number,
                    ));
                    target_keys_for_account.insert(account_key);
                }
                FetchKeyEnvelope::Storage(storage_key) => {
                    target_keys_for_header.insert(HeaderMemorizerKey::new(
                        storage_key.chain_id,
                        storage_key.block_number,
                    ));
                    target_keys_for_storage.insert(storage_key);
                }
                FetchKeyEnvelope::Tx(tx_key) => {
                    target_keys_for_header.insert(HeaderMemorizerKey::new(
                        tx_key.chain_id,
                        tx_key.block_number,
                    ));
                    target_keys_for_tx.insert(tx_key);
                }
                FetchKeyEnvelope::TxReceipt(tx_receipt_key) => {
                    target_keys_for_header.insert(HeaderMemorizerKey::new(
                        tx_receipt_key.chain_id,
                        tx_receipt_key.block_number,
                    ));
                    target_keys_for_tx_receipt.insert(tx_receipt_key);
                }
            }
        }

        // fetch proofs using keys and construct result
        let (headers, mmr_meta) = self.get_headers_from_keys(target_keys_for_header).await?;
        let mut accounts = self.get_accounts_from_keys(target_keys_for_account).await?;
        let (accounts_from_storage_key, storages) =
            self.get_storages_from_keys(target_keys_for_storage).await?;
        let transactions = self.get_txs_from_keys(target_keys_for_tx).await?;
        let transaction_receipts = self
            .get_tx_receipts_from_keys(target_keys_for_tx_receipt)
            .await?;
        accounts.extend(accounts_from_storage_key);
        let accounts_result: Vec<ProcessedAccount> = accounts.into_iter().collect();
        Ok(ProcessedBlockProofs {
            mmr_meta,
            headers: headers.into_iter().collect(),
            accounts: accounts_result,
            storages: storages.into_iter().collect(),
            transactions,
            transaction_receipts,
        })
    }

    async fn get_headers_from_keys(
        &self,
        keys: HashSet<HeaderMemorizerKey>,
    ) -> Result<(HashSet<ProcessedHeader>, MMRMeta), ProviderError> {
        let start_fetch = Instant::now();

        let block_range = keys.iter().map(|x| x.block_number).collect::<Vec<_>>();
        if block_range.is_empty() {
            return Err(ProviderError::FetchKeyError(
                "Block range is empty".to_string(),
            ));
        }
        let target_blocks_batch: Vec<Vec<BlockNumber>> = if block_range.len() == 1 {
            vec![block_range.clone()]
        } else {
            self._chunk_vec_blocks_for_indexer(block_range)
        };

        let mut fetched_headers_proofs: HashSet<ProcessedHeader> = HashSet::new();
        let mut mmr = None;

        let real_target_blocks = keys.iter().map(|x| x.block_number).collect::<HashSet<_>>();
        for target_blocks in target_blocks_batch {
            let (start_block, end_block) =
                (target_blocks[0], target_blocks[target_blocks.len() - 1]);

            let indexer_response = self
                .header_provider
                .get_headers_proof(start_block, end_block)
                .await?;

            // validate MMR among range of blocks
            match mmr {
                None => {
                    mmr = Some(indexer_response.mmr_meta);
                }
                Some(ref existing_mmr) if existing_mmr != &indexer_response.mmr_meta => {
                    return Err(ProviderError::MismatchedMMRMeta);
                }
                _ => {}
            }

            // filter only the keys that are in the real target blocks
            let keys_in_real_target_blocks = indexer_response
                .headers
                .into_iter()
                .filter(|(block_number, _)| real_target_blocks.contains(block_number))
                .map(|(_, header_proof)| {
                    ProcessedHeader::new(
                        header_proof.rlp_block_header,
                        header_proof.element_index,
                        header_proof.siblings_hashes,
                    )
                });

            fetched_headers_proofs.extend(keys_in_real_target_blocks);
        }

        let duration = start_fetch.elapsed();
        info!("Time taken (Headers Proofs Fetch): {:?}", duration);

        Ok((fetched_headers_proofs, mmr.unwrap().into()))
    }

    async fn get_accounts_from_keys(
        &self,
        keys: HashSet<AccountMemorizerKey>,
    ) -> Result<HashSet<ProcessedAccount>, ProviderError> {
        let mut fetched_accounts_proofs: HashSet<ProcessedAccount> = HashSet::new();
        let start_fetch = Instant::now();

        // group by address
        let mut address_to_block_range: HashMap<Address, Vec<BlockNumber>> = HashMap::new();
        for key in keys {
            let block_range = address_to_block_range.entry(key.address).or_default();
            block_range.push(key.block_number);
        }

        // loop through each address and fetch storages
        for (address, block_range) in address_to_block_range {
            if block_range.is_empty() {
                return Err(ProviderError::FetchKeyError(
                    "Block range is empty".to_string(),
                ));
            }
            let target_blocks_batch: Vec<Vec<BlockNumber>> = if block_range.len() == 1 {
                vec![block_range.clone()]
            } else {
                self._chunk_vec_blocks_for_mpt(block_range)
            };

            let mut account_mpt_proofs: Vec<ProcessedMPTProof> = vec![];
            for target_blocks in target_blocks_batch {
                let account_proofs = self
                    .rpc_provider
                    .get_account_proofs(target_blocks.clone(), address)
                    .await?;

                for block in target_blocks {
                    let account_proof = account_proofs.get(&block).unwrap().clone();
                    let account_proof = ProcessedMPTProof {
                        block_number: block,
                        proof: account_proof.account_proof,
                    };
                    account_mpt_proofs.push(account_proof);
                }
            }
            fetched_accounts_proofs.insert(ProcessedAccount::new(address, account_mpt_proofs));
        }
        let duration = start_fetch.elapsed();
        info!("Time taken (Accounts Proofs Fetch): {:?}", duration);

        Ok(fetched_accounts_proofs)
    }

    async fn get_storages_from_keys(
        &self,
        keys: HashSet<StorageMemorizerKey>,
    ) -> Result<(HashSet<ProcessedAccount>, HashSet<ProcessedStorage>), ProviderError> {
        let mut fetched_accounts_proofs: HashSet<ProcessedAccount> = HashSet::new();
        let mut fetched_storage_proofs: HashSet<ProcessedStorage> = HashSet::new();
        let start_fetch = Instant::now();

        // group by address and slot
        let mut address_slot_to_block_range: HashMap<(Address, B256), Vec<u64>> = HashMap::new();
        for key in keys {
            let block_range = address_slot_to_block_range
                .entry((key.address, key.key))
                .or_default();
            block_range.push(key.block_number);
        }

        // loop through each address and fetch storages
        for ((address, storage_slot), block_range) in address_slot_to_block_range {
            if block_range.is_empty() {
                return Err(ProviderError::FetchKeyError(
                    "Block range is empty".to_string(),
                ));
            }

            let target_blocks_batch: Vec<Vec<BlockNumber>> = if block_range.len() == 1 {
                vec![block_range.clone()]
            } else {
                self._chunk_vec_blocks_for_mpt(block_range)
            };

            let mut storage_mpt_proof: Vec<ProcessedMPTProof> = vec![];
            let mut account_mpt_proofs: Vec<ProcessedMPTProof> = vec![];
            for target_blocks in target_blocks_batch {
                let storage_proof = self
                    .rpc_provider
                    .get_storage_proofs(target_blocks.clone(), address, storage_slot)
                    .await?;

                for block in target_blocks {
                    let account_proof_response = storage_proof.get(&block).unwrap().clone();
                    account_mpt_proofs.push(ProcessedMPTProof {
                        block_number: block,
                        proof: account_proof_response.account_proof,
                    });
                    storage_mpt_proof.push(ProcessedMPTProof::new(
                        block,
                        account_proof_response.storage_proof[0].proof.clone(),
                    ));
                }
            }
            fetched_accounts_proofs.insert(ProcessedAccount::new(address, account_mpt_proofs));
            fetched_storage_proofs.insert(ProcessedStorage::new(
                address,
                storage_slot,
                storage_mpt_proof,
            ));
        }
        let duration = start_fetch.elapsed();
        info!("Time taken (Storage Proofs Fetch): {:?}", duration);

        Ok((fetched_accounts_proofs, fetched_storage_proofs))
    }

    pub async fn get_txs_from_keys(
        &self,
        keys: HashSet<TxMemorizerKey>,
    ) -> Result<Vec<ProcessedTransaction>, ProviderError> {
        let mut fetched_transactions = vec![];
        let start_fetch = Instant::now();

        // group by block number
        let mut block_to_tx_range: HashMap<BlockNumber, Vec<TxIndex>> = HashMap::new();
        for key in keys {
            let tx_range = block_to_tx_range.entry(key.block_number).or_default();
            tx_range.push(key.tx_index);
        }

        for (block_number, tx_range) in block_to_tx_range {
            let mut tx_trie_provider = TxsMptHandler::new(self.tx_provider_url.clone()).unwrap();
            loop {
                let trie_response = tx_trie_provider
                    .build_tx_tree_from_block(block_number)
                    .await;

                match trie_response {
                    Ok(_) => break,
                    Err(EthTrieError::RPC(RpcError::Transport(TransportErrorKind::HttpError(
                        http_error,
                    )))) if http_error.status == 429 => {
                        // retry if 429 error
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        continue;
                    }
                    _ => return Err(ProviderError::EthTrieError(trie_response.err().unwrap())),
                }
            }

            for tx_index in tx_range {
                let proof = tx_trie_provider
                    .get_proof(tx_index)
                    .unwrap()
                    .into_iter()
                    .map(Bytes::from)
                    .collect::<Vec<_>>();

                fetched_transactions.push(ProcessedTransaction::new(tx_index, block_number, proof));
            }
        }
        let duration = start_fetch.elapsed();
        info!("Time taken (Transaction Fetch): {:?}", duration);
        Ok(fetched_transactions)
    }

    pub async fn get_tx_receipts_from_keys(
        &self,
        keys: HashSet<TxReceiptMemorizerKey>,
    ) -> Result<Vec<ProcessedReceipt>, ProviderError> {
        let mut fetched_transaction_receipts = vec![];
        let start_fetch = Instant::now();
        // group by block number
        let mut block_to_tx_range: HashMap<BlockNumber, Vec<TxIndex>> = HashMap::new();
        for key in keys {
            let tx_range = block_to_tx_range.entry(key.block_number).or_default();
            tx_range.push(key.tx_index);
        }

        for (block_number, tx_range) in block_to_tx_range {
            let mut tx_receipt_trie_provider =
                TxReceiptsMptHandler::new(self.tx_provider_url.clone()).unwrap();
            loop {
                let trie_response = tx_receipt_trie_provider
                    .build_tx_receipts_tree_from_block(block_number)
                    .await;

                match trie_response {
                    Ok(_) => break,
                    Err(EthTrieError::RPC(RpcError::Transport(TransportErrorKind::HttpError(
                        http_error,
                    )))) if http_error.status == 429 => {
                        // retry if 429 error
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        continue;
                    }
                    _ => return Err(ProviderError::EthTrieError(trie_response.err().unwrap())),
                }
            }

            for tx_index in tx_range {
                let proof = tx_receipt_trie_provider
                    .get_proof(tx_index)
                    .unwrap()
                    .into_iter()
                    .map(Bytes::from)
                    .collect::<Vec<_>>();

                fetched_transaction_receipts.push(ProcessedReceipt::new(
                    tx_index,
                    block_number,
                    proof,
                ));
            }
        }
        let duration = start_fetch.elapsed();
        info!("Time taken (Transaction Receipts Fetch): {:?}", duration);
        Ok(fetched_transaction_receipts)
    }
}

#[cfg(test)]
mod tests {
    use alloy::primitives::address;
    use reqwest::Url;

    use super::*;
    use crate::evm::provider::EvmProvider;
    use crate::key::{AccountMemorizerKey, HeaderMemorizerKey};

    const SEPOLIA_RPC_URL: &str =
        "https://eth-sepolia.g.alchemy.com/v2/xar76cftwEtqTBWdF4ZFy9n8FLHAETDv";

    #[tokio::test]
    async fn test_fetch_headers_from_keys() {
        let target_chain_id = 11155111;
        let provider =
            EvmProvider::new_with_url(Url::parse(SEPOLIA_RPC_URL).unwrap(), target_chain_id);
        let keys = vec![
            FetchKeyEnvelope::Header(HeaderMemorizerKey::new(target_chain_id, 1)),
            FetchKeyEnvelope::Header(HeaderMemorizerKey::new(target_chain_id, 2)),
            FetchKeyEnvelope::Header(HeaderMemorizerKey::new(target_chain_id, 3)),
        ];
        let proofs = provider.fetch_proofs_from_keys(keys).await.unwrap();
        assert_eq!(proofs.headers.len(), 3);
    }

    #[tokio::test]
    async fn test_get_accounts_from_keys() {
        let target_chain_id = 11155111;
        let provider =
            EvmProvider::new_with_url(Url::parse(SEPOLIA_RPC_URL).unwrap(), target_chain_id);
        let target_address = address!("7f2c6f930306d3aa736b3a6c6a98f512f74036d4");
        let keys = vec![
            FetchKeyEnvelope::Account(AccountMemorizerKey::new(
                target_chain_id,
                6127485,
                target_address,
            )),
            FetchKeyEnvelope::Account(AccountMemorizerKey::new(
                target_chain_id,
                6127486,
                target_address,
            )),
            FetchKeyEnvelope::Account(AccountMemorizerKey::new(
                target_chain_id,
                6127487,
                target_address,
            )),
        ];
        let proofs = provider.fetch_proofs_from_keys(keys).await.unwrap();
        assert_eq!(proofs.accounts[0].proofs.len(), 3);
        assert_eq!(proofs.headers.len(), 3);
    }

    #[tokio::test]
    async fn test_complex_query_from_storage_keys() {
        let start_fetch = Instant::now();
        let target_chain_id = 11155111;
        let provider =
            EvmProvider::new_with_url(Url::parse(SEPOLIA_RPC_URL).unwrap(), target_chain_id);
        let target_address = address!("7f2c6f930306d3aa736b3a6c6a98f512f74036d4");
        let target_slot = B256::ZERO;
        let keys = vec![
            FetchKeyEnvelope::Storage(StorageMemorizerKey::new(
                target_chain_id,
                0,
                target_address,
                target_slot,
            )),
            FetchKeyEnvelope::Storage(StorageMemorizerKey::new(
                target_chain_id,
                6127486,
                target_address,
                target_slot,
            )),
            FetchKeyEnvelope::Storage(StorageMemorizerKey::new(
                target_chain_id,
                6127487,
                target_address,
                target_slot,
            )),
            FetchKeyEnvelope::Storage(StorageMemorizerKey::new(
                target_chain_id,
                4127497,
                target_address,
                target_slot,
            )),
            FetchKeyEnvelope::Storage(StorageMemorizerKey::new(
                target_chain_id,
                4127487,
                target_address,
                target_slot,
            )),
            FetchKeyEnvelope::Storage(StorageMemorizerKey::new(
                target_chain_id,
                4127477,
                target_address,
                target_slot,
            )),
        ];
        let proofs = provider.fetch_proofs_from_keys(keys).await.unwrap();
        let duration = start_fetch.elapsed();
        println!("Time taken (Total Proofs Fetch): {:?}", duration);
        assert_eq!(proofs.headers.len(), 6);
        assert_eq!(proofs.accounts[0].proofs.len(), 6);
        assert_eq!(proofs.storages[0].proofs.len(), 6);
    }
}
