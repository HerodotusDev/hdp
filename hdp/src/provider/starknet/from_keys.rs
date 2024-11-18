use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

use starknet_crypto::Felt;
use tracing::info;

use crate::{
    primitives::processed_types::{
        block_proofs::{convert_to_mmr_with_sn_headers, ProcessedBlockProofs, StarkNetBlockProofs},
        mmr::MMRMeta,
        starknet::{header::ProcessedHeader, storage::ProcessedStorage as SNProcessedStorage},
    },
    provider::{
        error::ProviderError,
        key::{CategorizedFetchKeys, StarknetHeaderKey, StarknetStorageKey},
    },
};

use super::provider::StarknetProvider;

impl StarknetProvider {
    /// This is the public entry point of provider.
    pub async fn fetch_proofs_from_keys(
        &self,
        fetch_keys: CategorizedFetchKeys,
    ) -> Result<ProcessedBlockProofs, ProviderError> {
        let chain_id = self.header_provider.from_chain_id.to_numeric_id();
        // fetch proofs using keys and construct result
        let mmr_with_headers = self.get_headers_from_keys(fetch_keys.sn_headers).await?;

        let storages = if fetch_keys.sn_storages.is_empty() {
            HashSet::new()
        } else {
            self.get_storages_from_keys(fetch_keys.sn_storages).await?
        };

        Ok(ProcessedBlockProofs::StarkNet(StarkNetBlockProofs {
            chain_id,
            mmr_with_headers: convert_to_mmr_with_sn_headers(mmr_with_headers),
            storages: storages.into_iter().collect(),
        }))
    }

    async fn get_headers_from_keys(
        &self,
        keys: HashSet<StarknetHeaderKey>,
    ) -> Result<HashMap<MMRMeta, HashSet<ProcessedHeader>>, ProviderError> {
        let start_fetch = Instant::now();
        let block_range = keys.iter().map(|x| x.block_number).collect::<Vec<_>>();

        if block_range.is_empty() {
            return Err(ProviderError::FetchKeyError(
                "Block range is empty".to_string(),
            ));
        }
        let target_blocks_batch: Vec<Vec<u64>> = if block_range.len() == 1 {
            vec![block_range]
        } else {
            self._chunk_vec_blocks_for_indexer(block_range)
        };

        let mut fetched_headers_proofs: HashMap<MMRMeta, HashSet<ProcessedHeader>> = HashMap::new();

        let real_target_blocks = keys.iter().map(|x| x.block_number).collect::<HashSet<_>>();
        for target_blocks in target_blocks_batch {
            let (start_block, end_block) =
                (target_blocks[0], target_blocks[target_blocks.len() - 1]);

            let indexer_response = self
                .header_provider
                .get_headers_proof(start_block, end_block)
                .await?;

            // filter only the keys that are in the real target blocks and create ProcessedHeaders
            let keys_in_real_target_blocks: Vec<ProcessedHeader> = indexer_response
                .headers
                .into_iter()
                .filter(|(block_number, _)| real_target_blocks.contains(block_number))
                .map(|(_, header_proof)| {
                    ProcessedHeader::new(
                        header_proof.block_header.get_sn_block_header().fields,
                        header_proof.element_index,
                        header_proof.siblings_hashes,
                    )
                })
                .collect();

            let fetched_mmr = indexer_response.mmr_meta;
            let mmr_meta = MMRMeta::from_indexer(fetched_mmr);
            fetched_headers_proofs
                .entry(mmr_meta)
                .and_modify(|existing_headers| {
                    existing_headers.extend(keys_in_real_target_blocks.iter().cloned());
                })
                .or_insert_with(|| keys_in_real_target_blocks.into_iter().collect());
        }

        let duration = start_fetch.elapsed();
        info!("time taken (Headers Proofs Fetch): {:?}", duration);

        if !fetched_headers_proofs.is_empty() {
            Ok(fetched_headers_proofs)
        } else {
            Err(ProviderError::MmrNotFound)
        }
    }

    async fn get_storages_from_keys(
        &self,
        keys: HashSet<StarknetStorageKey>,
    ) -> Result<HashSet<SNProcessedStorage>, ProviderError> {
        let mut fetched_storage_proofs: HashSet<SNProcessedStorage> = HashSet::new();
        let start_fetch = Instant::now();

        // group by address => block range => storage keys
        let mut address_to_block_range_storage_keys: HashMap<Felt, HashMap<u64, Vec<Felt>>> =
            HashMap::new();
        let chain_id = keys.iter().map(|x| x.chain_id).next().unwrap();
        for key in keys {
            let mapped_value = address_to_block_range_storage_keys
                .entry(key.address)
                .or_default();
            let storage_keys = mapped_value.entry(key.block_number).or_default();
            storage_keys.push(key.key);
        }

        // loop through each address and chunk fetch requests
        for (address, block_storage_key_map) in address_to_block_range_storage_keys {
            if block_storage_key_map.is_empty() {
                return Err(ProviderError::FetchKeyError(
                    "Block range is empty".to_string(),
                ));
            }
            let target_blocks_batch: Vec<Vec<(u64, Vec<Felt>)>> =
                if block_storage_key_map.len() == 1 {
                    vec![block_storage_key_map.into_iter().collect()]
                } else {
                    self._chunk_vec_blocks_keys(block_storage_key_map.into_iter().collect())
                };

            for target in target_blocks_batch {
                let storage_proof = self
                    .rpc_provider
                    .get_proofs(target.clone(), address)
                    .await?;

                for (block_number, storage_keys) in target {
                    let proof = storage_proof.get(&block_number).unwrap();
                    let storage_proof_of_block = SNProcessedStorage::new(
                        chain_id,
                        block_number,
                        address,
                        storage_keys,
                        proof.clone(),
                    );

                    fetched_storage_proofs.insert(storage_proof_of_block);
                }
            }
        }

        let duration = start_fetch.elapsed();
        info!("time taken (Storages Proofs Fetch): {:?}", duration);
        Ok(fetched_storage_proofs)
    }
}

#[cfg(test)]
#[cfg(feature = "test_utils")]
mod tests {
    use super::*;
    use crate::provider::key::{categorize_fetch_keys, FetchKeyEnvelope};
    use dotenv::dotenv;
    use starknet_crypto::Felt;
    use std::str::FromStr;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn initialize() {
        INIT.call_once(|| {
            dotenv().ok();
        });
    }

    #[tokio::test]
    #[cfg(feature = "test_utils")]
    async fn test_proofs_from_storage_keys() {
        initialize();
        let start_fetch = Instant::now();
        let target_chain_id = crate::primitives::ChainId::StarknetSepolia;
        let provider = StarknetProvider::default();
        let target_address =
            Felt::from_str("0x017E2D0662675DD83B4B58A0A659EAFA131FDD01FA6DABD5002D8815DD2D17A5")
                .unwrap();
        let target_slot =
            Felt::from_str("0x032ce6490b615c86e31587e14d6140e5a46231d9b8bf870fd708d71140c3ed2f")
                .unwrap();
        let keys = vec![
            FetchKeyEnvelope::StarknetStorage(StarknetStorageKey::new(
                target_chain_id,
                208473,
                target_address,
                target_slot,
            )),
            FetchKeyEnvelope::StarknetStorage(StarknetStorageKey::new(
                target_chain_id,
                208483,
                target_address,
                target_slot,
            )),
            FetchKeyEnvelope::StarknetStorage(StarknetStorageKey::new(
                target_chain_id,
                208383,
                target_address,
                target_slot,
            )),
            FetchKeyEnvelope::StarknetStorage(StarknetStorageKey::new(
                target_chain_id,
                208384,
                target_address,
                target_slot,
            )),
            FetchKeyEnvelope::StarknetStorage(StarknetStorageKey::new(
                target_chain_id,
                208385,
                target_address,
                target_slot,
            )),
            FetchKeyEnvelope::StarknetStorage(StarknetStorageKey::new(
                target_chain_id,
                208386,
                target_address,
                target_slot,
            )),
        ];
        let (chain_id, fetched_keys) = categorize_fetch_keys(keys).into_iter().next().unwrap();
        assert_eq!(chain_id, target_chain_id);
        let proofs = provider
            .fetch_proofs_from_keys(fetched_keys)
            .await
            .unwrap()
            .get_starknet_proofs()
            .unwrap();
        let duration = start_fetch.elapsed();
        println!("Time taken (Total Proofs Fetch): {:?}", duration);
        assert_eq!(proofs.mmr_with_headers[0].headers.len(), 6);
    }
}
