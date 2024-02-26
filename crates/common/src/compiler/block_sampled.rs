use std::{str::FromStr, sync::Arc};

use crate::{
    block::{
        account::{decode_account_field, AccountField},
        header::{decode_header_field, HeaderField},
    },
    datalake::base::{
        AccountResult, BlockResult, DatalakeResult, MMRMetaResult, MMRResult, StorageResult,
    },
    fetcher::{memory::StoredHeader, AbstractFetcher},
};
use anyhow::Result;
use reth_primitives::revm_primitives::bitvec::vec;
use tokio::sync::RwLock;

pub async fn compile_block_sampled_datalake(
    block_range_start: u64,
    block_range_end: u64,
    sampled_property: &str,
    increment: u64,
    fetcher: Arc<RwLock<AbstractFetcher>>,
) -> Result<DatalakeResult> {
    let mut abstract_fetcher = fetcher.write().await;
    let property_parts: Vec<&str> = sampled_property.split('.').collect();
    let collection = property_parts[0];

    let mut aggregation_set: Vec<String> = Vec::new();
    let target_block_range: Vec<u64> = (block_range_start..=block_range_end)
        .step_by(increment as usize)
        .collect();

    let full_header_and_proof_result = abstract_fetcher
        .get_full_header_with_proof(target_block_range.clone())
        .await?;
    let mmr_meta = full_header_and_proof_result.1;
    let mut blocks: Vec<BlockResult> = vec![];

    match collection {
        "header" => {
            let property = property_parts[1];

            for block in target_block_range {
                let fetched_block = full_header_and_proof_result.0.get(&block).unwrap().clone();
                let account = None;
                let value = decode_header_field(
                    &fetched_block.0,
                    HeaderField::from_str(&property.to_uppercase()).unwrap(),
                );

                blocks.push(BlockResult {
                    leaf_idx: fetched_block.2,
                    mmr_proof: fetched_block.1,
                    rlp_encoded_header: fetched_block.0,
                    account,
                });

                aggregation_set.push(value);
            }
        }
        "account" => {
            let account = property_parts[1];
            let property = property_parts[2];

            for i in block_range_start..=block_range_end {
                if i % increment != 0 {
                    continue;
                }
                let fetched_block = full_header_and_proof_result.0.get(&i).unwrap().clone();
                let acc = abstract_fetcher
                    .get_rlp_account(i, account.to_string())
                    .await;

                let value = decode_account_field(
                    &acc,
                    AccountField::from_str(&property.to_uppercase()).unwrap(),
                );

                blocks.push(BlockResult {
                    leaf_idx: fetched_block.2,
                    mmr_proof: fetched_block.1,
                    rlp_encoded_header: fetched_block.0,
                    account: Some(AccountResult {
                        address: account.to_string(),
                        // TODO: account_proof
                        account_proof: vec![],
                        rlp_encoded_account: acc,
                        storage: None,
                    }),
                });

                aggregation_set.push(value);
            }
        }
        "storage" => {
            let account = property_parts[1];
            let slot = property_parts[2];

            for i in block_range_start..=block_range_end {
                if i % increment != 0 {
                    continue;
                }
                let fetched_block = full_header_and_proof_result.0.get(&i).unwrap().clone();

                let value = abstract_fetcher
                    .get_storage_value(i, account.to_string(), slot.to_string())
                    .await;

                blocks.push(BlockResult {
                    leaf_idx: fetched_block.2,
                    mmr_proof: fetched_block.1,
                    rlp_encoded_header: fetched_block.0,
                    account: Some(AccountResult {
                        address: account.to_string(),
                        // TODO: account_proof
                        account_proof: vec![],
                        //TODO
                        rlp_encoded_account: "account".to_string(),
                        storage: Some(StorageResult {
                            // TODO: storage_proof
                            storage_proof: vec![],
                            storage_key: slot.to_string(),
                            storage_value: value.clone(),
                        }),
                    }),
                });
                aggregation_set.push(value);
            }
        }
        _ => todo!(),
    }

    Ok(DatalakeResult {
        mmr: vec![MMRResult {
            compiled_result: aggregation_set,
            blocks,
            mmr_meta,
        }],
    })
}
