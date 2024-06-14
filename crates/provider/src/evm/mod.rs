use alloy_primitives::{keccak256, Bytes};
use anyhow::Result;
use eth_trie_proofs::{tx_receipt_trie::TxReceiptsMptHandler, tx_trie::TxsMptHandler};
use rpc::{FetchedTransactionProof, FetchedTransactionReceiptProof};
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    time::Instant,
    vec,
};

use tokio::sync::mpsc;
use tracing::{error, info};

use hdp_primitives::{
    block::header::Header as HeaderPrimitive,
    datalake::{
        block_sampled::output::{Account, Storage},
        output::{Header, HeaderProof, MMRMeta, MPTProof},
        transactions::output::{Transaction, TransactionReceipt},
    },
    utils::tx_index_to_tx_key,
};

use crate::key::{
    AccountProviderKey, FetchKeyEnvelope, HeaderProviderKey, StorageProviderKey, TxProviderKey,
    TxReceiptProviderKey,
};

use self::{
    memory::{InMemoryProvider, RlpEncodedValue, StoredHeader, StoredHeaders},
    rpc::{FetchedAccountProof, FetchedStorageAccountProof, HeaderProvider, TrieProofProvider},
};

pub(crate) mod memory;
pub(crate) mod rpc;

// For more information swagger doc: https://rs-indexer.api.herodotus.cloud/swagger
const HERODOTUS_RS_INDEXER_URL: &str = "https://rs-indexer.api.herodotus.cloud/accumulators";

/// [`AbstractProvider`] abstracts the fetching of data from the RPC and memory.
///  It uses a [`InMemoryProvider`] and a [`RpcProvider`] to fetch data.
///
/// but handle requests so that it would not make duplicate requests
pub struct AbstractProvider {
    /// [`InMemoryProvider`] is used to fetch data from memory.
    // TODO: It's not using for now
    memory: InMemoryProvider,
    /// Fetch data from the RPC
    trie_proof_provider: TrieProofProvider,
    /// Fetch block headers and MMR data from the Herodotus indexer.
    header_provider: HeaderProvider,
}

pub struct AbstractProviderConfig {
    pub rpc_url: &'static str,
    pub chain_id: u64,
    pub rpc_chunk_size: u64,
}

/// Provider should fetch all the proofs and rlp values from given keys.
#[derive(Serialize)]
pub struct AbstractProviderResult {
    pub mmr_meta: MMRMeta,
    pub headers: Vec<Header>,
    pub accounts: Vec<Account>,
    pub storages: Vec<Storage>,
    pub transactions: Vec<Transaction>,
    pub transaction_receipts: Vec<TransactionReceipt>,
}

impl AbstractProvider {
    pub fn new(config: AbstractProviderConfig) -> Self {
        Self {
            memory: InMemoryProvider::new(),
            trie_proof_provider: TrieProofProvider::new(config.rpc_url, config.rpc_chunk_size),
            header_provider: HeaderProvider::new(HERODOTUS_RS_INDEXER_URL, config.chain_id),
        }
    }

    /// This is the public entry point of provider.  
    pub async fn fetch_proofs_from_keys(
        &self,
        fetch_keys: HashSet<FetchKeyEnvelope>,
    ) -> Result<AbstractProviderResult> {
        // categorize fetch keys
        let mut target_keys_for_header = vec![];
        let mut target_keys_for_account = vec![];
        let mut target_keys_for_storage = vec![];
        let mut target_keys_for_tx = vec![];
        let mut target_keys_for_tx_receipt = vec![];
        for key in fetch_keys {
            match key {
                FetchKeyEnvelope::Header(header_key) => {
                    target_keys_for_header.push(header_key);
                }
                FetchKeyEnvelope::Account(account_key) => {
                    target_keys_for_account.push(account_key);
                }
                FetchKeyEnvelope::Storage(storage_key) => {
                    target_keys_for_storage.push(storage_key);
                }
                FetchKeyEnvelope::Tx(tx_key) => {
                    target_keys_for_tx.push(tx_key);
                }
                FetchKeyEnvelope::TxReceipt(tx_receipt_key) => {
                    target_keys_for_tx_receipt.push(tx_receipt_key);
                }
            }
        }

        // fetch proofs using keys and construct result
        let (headers, mmr_meta) = self
            .fetch_headers_from_keys(&target_keys_for_header)
            .await?;
        let accounts = self
            .get_accounts_from_keys(&target_keys_for_account)
            .await?;
        let storages = self
            .get_storages_from_keys(&target_keys_for_storage)
            .await?;
        let transactions = self.get_txs_from_keys(&target_keys_for_tx).await?;
        let transaction_receipts = self
            .get_tx_receipts_from_keys(&target_keys_for_tx_receipt)
            .await?;

        Ok(AbstractProviderResult {
            mmr_meta,
            headers,
            accounts,
            storages,
            transactions,
            transaction_receipts,
        })
    }

    pub async fn fetch_headers_from_keys(
        &self,
        keys: &[HeaderProviderKey],
    ) -> Result<(Vec<Header>, MMRMeta)> {
        let mut result_headers: Vec<Header> = vec![];
        // Fetch MMR data and header data from Herodotus indexer
        let start_fetch = Instant::now();

        let start_block = keys.iter().map(|x| x.block_number).min().unwrap();
        let end_block = keys.iter().map(|x| x.block_number).max().unwrap();

        let mmr_data = self
            .header_provider
            .get_sequencial_headers_and_mmr_from_indexer(start_block, end_block)
            .await;

        match mmr_data {
            Ok(mmr) => {
                info!("Successfully fetched MMR data from indexer");
                let duration = start_fetch.elapsed();
                info!("Time taken (fetch from Indexer): {:?}", duration);
                for block_proof in &mmr.1 {
                    result_headers.push(Header {
                        rlp: block_proof.1.rlp_block_header.value.clone(),
                        proof: HeaderProof {
                            leaf_idx: block_proof.1.element_index,
                            mmr_path: block_proof.1.siblings_hashes.clone(),
                        },
                    });
                }

                Ok((
                    result_headers,
                    MMRMeta {
                        id: mmr.0.mmr_id,
                        root: mmr.0.mmr_root,
                        size: mmr.0.mmr_size,
                        peaks: mmr.0.mmr_peaks,
                    },
                ))
            }
            Err(e) => {
                let duration = start_fetch.elapsed();
                info!("Time taken (during from Indexer): {:?}", duration);
                error!(
                    "Something went wrong while fetching MMR data from indexer: {}",
                    e
                );
                Err(e)
            }
        }
    }

    pub async fn get_accounts_from_keys(
        &self,
        keys: &[AccountProviderKey],
    ) -> Result<Vec<Account>> {
        let start_fetch = Instant::now();

        // group by address
        let mut address_to_block_range: HashMap<String, Vec<u64>> = HashMap::new();
        for key in keys {
            let block_range = address_to_block_range
                .entry(key.address.to_string())
                .or_default();
            block_range.push(key.block_number);
        }
        // loop through each address and fetch accounts
        let mut accounts = vec![];
        for (address, block_range) in address_to_block_range {
            let (rpc_sender, mut rx) = mpsc::channel::<FetchedAccountProof>(32);

            self.trie_proof_provider
                .get_account_proofs(rpc_sender, block_range, &address)
                .await;

            let mut account_proofs: Vec<MPTProof> = vec![];

            while let Some(proof) = rx.recv().await {
                let account_proof = MPTProof {
                    block_number: proof.block_number,
                    proof: proof.account_proof,
                };
                account_proofs.push(account_proof);
            }
            let account_key = keccak256(address.clone());
            let account = Account {
                address,
                account_key: account_key.to_string(),
                proofs: account_proofs,
            };
            accounts.push(account);
        }
        let duration = start_fetch.elapsed();
        info!("Time taken (Account Fetch): {:?}", duration);

        Ok(accounts)
    }

    pub async fn get_storages_from_keys(
        &self,
        keys: &[StorageProviderKey],
    ) -> Result<Vec<Storage>> {
        let start_fetch = Instant::now();

        // group by address and slot
        let mut address_slot_to_block_range: HashMap<(String, String), Vec<u64>> = HashMap::new();
        for key in keys {
            let block_range = address_slot_to_block_range
                .entry((key.address.to_string(), key.key.to_string()))
                .or_default();
            block_range.push(key.block_number);
        }
        // loop through each address and fetch storages
        let mut storages = vec![];
        for ((address, slot), block_range) in address_slot_to_block_range {
            let (rpc_sender, mut rx) = mpsc::channel::<FetchedStorageAccountProof>(32);

            self.trie_proof_provider
                .get_storage_proofs(rpc_sender, block_range, &address, slot.clone())
                .await;

            let mut storage_proofs: Vec<MPTProof> = vec![];

            while let Some(proof) = rx.recv().await {
                let storage_proof = MPTProof {
                    block_number: proof.block_number,
                    proof: proof.storage_proof,
                };
                storage_proofs.push(storage_proof);
            }
            let storage_key = keccak256(slot.clone()).to_string();
            let storage = Storage {
                address,
                slot,
                storage_key,
                proofs: storage_proofs,
            };
            storages.push(storage);
        }
        let duration = start_fetch.elapsed();
        info!("Time taken (Storage Fetch): {:?}", duration);

        Ok(storages)
    }

    pub async fn get_txs_from_keys(&self, keys: &[TxProviderKey]) -> Result<Vec<Transaction>> {
        let start_fetch = Instant::now();
        // group by block number
        let mut block_to_tx_range: HashMap<u64, Vec<u64>> = HashMap::new();
        for key in keys {
            let tx_range = block_to_tx_range.entry(key.block_number).or_default();
            tx_range.push(key.tx_index);
        }

        let mut transactions = vec![];
        for (block_number, tx_range) in block_to_tx_range {
            let mut txs_mpt_handler = TxsMptHandler::new(self.trie_proof_provider.url).unwrap();
            txs_mpt_handler
                .build_tx_tree_from_block(block_number)
                .await
                .unwrap();
            let txs = txs_mpt_handler.get_elements().unwrap();

            for tx_index in tx_range {
                let proof = txs_mpt_handler
                    .get_proof(tx_index)
                    .unwrap()
                    .into_iter()
                    .map(|x| Bytes::from(x).to_string())
                    .collect::<Vec<_>>();
                let consensus_tx = txs[tx_index as usize].clone();
                let rlp = Bytes::from(consensus_tx.rlp_encode()).to_string();
                let key_fixed_bytes = tx_index_to_tx_key(tx_index);
                let tx = Transaction {
                    block_number,
                    proof,
                    key: key_fixed_bytes,
                };
                transactions.push(tx);
            }
        }
        let duration = start_fetch.elapsed();
        info!("Time taken (Transaction Fetch): {:?}", duration);
        Ok(transactions)
    }

    pub async fn get_tx_receipts_from_keys(
        &self,
        keys: &[TxReceiptProviderKey],
    ) -> Result<Vec<TransactionReceipt>> {
        let start_fetch = Instant::now();
        // group by block number
        let mut block_to_tx_receipt_range: HashMap<u64, Vec<u64>> = HashMap::new();
        for key in keys {
            let tx_receipt_range = block_to_tx_receipt_range
                .entry(key.block_number)
                .or_default();
            tx_receipt_range.push(key.tx_index);
        }

        let mut transaction_receipts = vec![];
        for (block_number, tx_receipt_range) in block_to_tx_receipt_range {
            let mut tx_reciepts_mpt_handler =
                TxReceiptsMptHandler::new(self.trie_proof_provider.url).unwrap();

            tx_reciepts_mpt_handler
                .build_tx_receipts_tree_from_block(block_number)
                .await
                .unwrap();
            let tx_receipts = tx_reciepts_mpt_handler.get_elements().unwrap();

            for tx_receipt_index in tx_receipt_range {
                let proof = tx_reciepts_mpt_handler
                    .get_proof(tx_receipt_index)
                    .unwrap()
                    .into_iter()
                    .map(|x| Bytes::from(x).to_string())
                    .collect::<Vec<_>>();
                let consensus_tx_receipt = tx_receipts[tx_receipt_index as usize].clone();
                let rlp = Bytes::from(consensus_tx_receipt.rlp_encode()).to_string();
                let key_fixed_bytes = tx_index_to_tx_key(tx_receipt_index);
                let tx_receipt = TransactionReceipt {
                    block_number,
                    proof,
                    key: key_fixed_bytes,
                };
                transaction_receipts.push(tx_receipt);
            }
        }

        let duration = start_fetch.elapsed();
        info!("Time taken (Transaction Receipt Fetch): {:?}", duration);
        Ok(transaction_receipts)
    }

    // TODO: wip
    pub async fn get_sequencial_full_header_with_proof(
        &self,
        start_block: u64,
        end_block: u64,
    ) -> Result<(StoredHeaders, MMRMeta)> {
        //? A map of block numbers to a boolean indicating whether the block was fetched.
        let mut blocks_map: HashMap<u64, StoredHeader> = HashMap::new();

        // Fetch MMR data and header data from Herodotus indexer
        let start_fetch = Instant::now();

        let mmr_data = self
            .header_provider
            .get_sequencial_headers_and_mmr_from_indexer(start_block, end_block)
            .await;

        match mmr_data {
            Ok(mmr) => {
                info!("Successfully fetched MMR data from indexer");
                let duration = start_fetch.elapsed();
                info!("Time taken (fetch from Indexer): {:?}", duration);
                for block_proof in &mmr.1 {
                    blocks_map.insert(
                        *block_proof.0,
                        (
                            block_proof.1.rlp_block_header.value.clone(),
                            block_proof.1.siblings_hashes.clone(),
                            block_proof.1.element_index,
                            mmr.0.mmr_id,
                        ),
                    );
                }

                Ok((
                    blocks_map,
                    MMRMeta {
                        id: mmr.0.mmr_id,
                        root: mmr.0.mmr_root,
                        size: mmr.0.mmr_size,
                        peaks: mmr.0.mmr_peaks,
                    },
                ))
            }
            Err(e) => {
                let duration = start_fetch.elapsed();
                info!("Time taken (during from Indexer): {:?}", duration);
                error!(
                    "Something went wrong while fetching MMR data from indexer: {}",
                    e
                );
                Err(e)
            }
        }
    }

    // Unoptimized version of get_rlp_header, just for testing purposes
    pub async fn get_rlp_header(&mut self, block_number: u64) -> RlpEncodedValue {
        match self.memory.get_rlp_header(block_number) {
            Some(header) => header,
            None => {
                let header_rpc = self
                    .trie_proof_provider
                    .get_block_by_number(block_number)
                    .await
                    .unwrap();
                let block_header = HeaderPrimitive::from(&header_rpc);
                let rlp_encoded = block_header.rlp_encode();
                self.memory.set_header(block_number, rlp_encoded.clone());

                rlp_encoded
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
    ) -> Result<HashMap<u64, FetchedAccountProof>> {
        let start_fetch = Instant::now();

        let target_block_range: Vec<u64> = (block_range_start..=block_range_end)
            .step_by(increment as usize)
            .collect();

        let (rpc_sender, mut rx) = mpsc::channel::<FetchedAccountProof>(32);

        self.trie_proof_provider
            .get_account_proofs(rpc_sender, target_block_range, &address)
            .await;

        let mut result = HashMap::new();

        while let Some(proof) = rx.recv().await {
            result.insert(proof.block_number, proof);
        }

        let duration = start_fetch.elapsed();
        info!("Time taken (Account Fetch): {:?}", duration);

        Ok(result)
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
    ) -> Result<HashMap<u64, FetchedStorageAccountProof>> {
        let start_fetch = Instant::now();
        //? A map of block numbers to a boolean indicating whether the block was fetched.
        let target_block_range: Vec<u64> = (block_range_start..=block_range_end)
            .step_by(increment as usize)
            .collect();

        let (rpc_sender, mut rx) = mpsc::channel::<FetchedStorageAccountProof>(32);
        self.trie_proof_provider
            .get_storage_proofs(rpc_sender, target_block_range, &address, slot)
            .await;

        let mut result = HashMap::new();

        while let Some(proof) = rx.recv().await {
            result.insert(proof.block_number, proof);
        }
        let duration = start_fetch.elapsed();
        info!("Time taken (Storage Fetch): {:?}", duration);

        Ok(result)
    }

    /// Fetches the encoded transaction with proof from the MPT trie for the given block number.
    /// The transaction is fetched from the MPT trie and the proof is generated from the MPT trie.
    pub async fn get_tx_with_proof_from_block(
        &self,
        target_block: u64,
        start_index: u64,
        end_index: u64,
        incremental: u64,
    ) -> Result<Vec<FetchedTransactionProof>> {
        let mut tx_with_proof = vec![];
        let mut txs_mpt_handler = TxsMptHandler::new(self.trie_proof_provider.url).unwrap();
        txs_mpt_handler
            .build_tx_tree_from_block(target_block)
            .await
            .unwrap();
        let txs = txs_mpt_handler.get_elements().unwrap();

        let target_tx_index_range = (start_index..end_index).step_by(incremental as usize);
        for tx_index in target_tx_index_range {
            let proof = txs_mpt_handler
                .get_proof(tx_index)
                .unwrap()
                .into_iter()
                .map(|x| Bytes::from(x).to_string())
                .collect::<Vec<_>>();
            let consensus_tx = txs[tx_index as usize].clone();
            let rlp = Bytes::from(consensus_tx.rlp_encode()).to_string();
            let tx_type = consensus_tx.0.tx_type() as u8;
            let fetched_result = FetchedTransactionProof {
                block_number: target_block,
                tx_index,
                encoded_transaction: rlp,
                transaction_proof: proof,
                tx_type,
            };
            tx_with_proof.push(fetched_result);
        }

        Ok(tx_with_proof)
    }

    /// Fetches the encoded transaction receipt with proof from the MPT trie for the given block number.
    /// The transaction receipt is fetched from the MPT trie and the proof is generated from the MPT trie.
    pub async fn get_tx_receipt_with_proof_from_block(
        &self,
        target_block: u64,
        start_index: u64,
        end_index: u64,
        incremental: u64,
    ) -> Result<Vec<FetchedTransactionReceiptProof>> {
        let mut tx_receipt_with_proof = vec![];
        let mut tx_reciepts_mpt_handler =
            TxReceiptsMptHandler::new(self.trie_proof_provider.url).unwrap();

        tx_reciepts_mpt_handler
            .build_tx_receipts_tree_from_block(target_block)
            .await
            .unwrap();
        let tx_receipts = tx_reciepts_mpt_handler.get_elements().unwrap();
        let target_tx_receipt_index_range = (start_index..end_index).step_by(incremental as usize);

        for tx_receipt_index in target_tx_receipt_index_range {
            let proof = tx_reciepts_mpt_handler
                .get_proof(tx_receipt_index)
                .unwrap()
                .into_iter()
                .map(|x| Bytes::from(x).to_string())
                .collect::<Vec<_>>();
            let consensus_tx_receipt = tx_receipts[tx_receipt_index as usize].clone();
            let rlp = Bytes::from(consensus_tx_receipt.rlp_encode()).to_string();
            let tx_receipt_type = consensus_tx_receipt.0.tx_type() as u8;
            tx_receipt_with_proof.push(FetchedTransactionReceiptProof {
                block_number: target_block,
                tx_index: tx_receipt_index,
                encoded_receipt: rlp,
                receipt_proof: proof,
                tx_type: tx_receipt_type,
            });
        }

        Ok(tx_receipt_with_proof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{hex, keccak256};

    fn rlp_string_to_block_hash(rlp_string: &str) -> String {
        keccak256(hex::decode(rlp_string).unwrap()).to_string()
    }

    // Non-paid personal alchemy endpoint
    const SEPOLIA_RPC_URL: &str =
        "https://eth-sepolia.g.alchemy.com/v2/xar76cftwEtqTBWdF4ZFy9n8FLHAETDv";

    #[tokio::test]
    async fn test_provider_get_rlp_header() {
        let config = AbstractProviderConfig {
            rpc_url: SEPOLIA_RPC_URL,
            chain_id: 11155111,
            rpc_chunk_size: 40,
        };
        let mut provider = AbstractProvider::new(config);
        let rlp_header = provider.get_rlp_header(0).await;
        let block_hash = rlp_string_to_block_hash(&rlp_header);
        assert_eq!(
            block_hash,
            "0x25a5cc106eea7138acab33231d7160d69cb777ee0c2c553fcddf5138993e6dd9"
        );
        let rlp_header = provider.get_rlp_header(5521772).await;
        let block_hash = rlp_string_to_block_hash(&rlp_header);
        assert_eq!(
            block_hash,
            "0xe72515bc74912f67912a64a458e6f2cd2742f8dfe0666e985749483dab0b7b9a"
        );
        let rlp_header = provider.get_rlp_header(487680).await;
        let block_hash = rlp_string_to_block_hash(&rlp_header);
        assert_eq!(
            block_hash,
            "0xf494127d30817d04b634eae9f6139d8155ee4c78ba60a35bd7be187378e93d6e"
        );
    }

    #[tokio::test]
    async fn get_block_range_from_nonce_range_non_constant() {
        let config = AbstractProviderConfig {
            rpc_url: SEPOLIA_RPC_URL,
            chain_id: 11155111,
            rpc_chunk_size: 40,
        };
        let provider = AbstractProvider::new(config);
        let block_range = provider
            .get_tx_with_proof_from_block(5530433, 10, 100, 1)
            .await
            .unwrap();

        assert_eq!(block_range.len(), 90);
        assert_eq!(block_range[0].encoded_transaction,"0xf873830beeeb84faa6fd50830148209447b854ad2ddb01cfee0b07f4e2da0ac50277b1168806f05b59d3b20000808401546d72a06af2b103dfb7bccc757d575bc11c38f2ecd1a22ca2fcf95a602119582c607927a047329735997e3357dfd7d63eda024d35f7012855aa12ba210f9ed311a517b5e6");

        let block_range = provider
            .get_tx_with_proof_from_block(5530433, 10, 100, 3)
            .await
            .unwrap();

        assert_eq!(block_range.len(), 30);
    }
}
