use hex::FromHex;
use std::{str::FromStr, sync::Arc};

use crate::datalake::base::DatalakeResult;
use alloy_primitives::{hex, keccak256};
use anyhow::{bail, Result};
use hdp_primitives::{
    block::{
        account::{decode_account_field, AccountField},
        header::{decode_header_field, HeaderField},
    },
    format::{Account, Header, HeaderProof, MPTProof, Storage},
};
use hdp_provider::evm::AbstractFetcher;
use tokio::sync::RwLock;

pub async fn compile_block_sampled_datalake(
    block_range_start: u64,
    block_range_end: u64,
    sampled_property: &str,
    increment: u64,
    fetcher: &Arc<RwLock<AbstractFetcher>>,
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
    let mut headers: Vec<Header> = vec![];
    let mut accounts: Vec<Account> = vec![];
    let mut storages: Vec<Storage> = vec![];

    match collection {
        "header" => {
            let property = property_parts[1];

            for block in target_block_range {
                let fetched_block = full_header_and_proof_result.0.get(&block).unwrap().clone();

                let value = decode_header_field(
                    &fetched_block.0,
                    HeaderField::from_str(&property.to_uppercase()).unwrap(),
                );

                headers.push(Header {
                    rlp: fetched_block.0,
                    proof: HeaderProof {
                        leaf_idx: fetched_block.2,
                        mmr_path: fetched_block.1,
                    },
                });

                aggregation_set.push(value);
            }
        }
        "account" => {
            let address = property_parts[1];
            let property = property_parts[2];

            let accounts_and_proofs_result = abstract_fetcher
                .get_range_account_with_proof(
                    block_range_start,
                    block_range_end,
                    increment,
                    address.to_string(),
                )
                .await?;

            let mut account_proofs: Vec<MPTProof> = vec![];
            // let mut encoded_account = "".to_string();

            for i in block_range_start..=block_range_end {
                if i % increment != 0 {
                    continue;
                }
                let fetched_block = full_header_and_proof_result.0.get(&i).unwrap().clone();
                let acc = accounts_and_proofs_result.get(&i).unwrap().clone();
                // encoded_account = acc.0.clone();

                let value = decode_account_field(
                    &acc.0,
                    AccountField::from_str(&property.to_uppercase()).unwrap(),
                );

                headers.push(Header {
                    rlp: fetched_block.0,
                    proof: HeaderProof {
                        leaf_idx: fetched_block.2,
                        mmr_path: fetched_block.1,
                    },
                });

                let account_proof = MPTProof {
                    block_number: i,
                    proof: acc.1,
                };

                account_proofs.push(account_proof);

                aggregation_set.push(value);
            }

            let address_bytes = Vec::from_hex(address).expect("Invalid hex string");
            let account_key = keccak256(address_bytes);

            accounts.push(Account {
                address: address.to_string(),
                account_key: account_key.to_string(),
                proofs: account_proofs,
            });
        }
        "storage" => {
            let address = property_parts[1];
            let slot = property_parts[2];

            let storages_and_proofs_result = abstract_fetcher
                .get_range_storage_with_proof(
                    block_range_start,
                    block_range_end,
                    increment,
                    address.to_string(),
                    slot.to_string(),
                )
                .await?;

            let mut storage_proofs: Vec<MPTProof> = vec![];
            let mut account_proofs: Vec<MPTProof> = vec![];

            for i in block_range_start..=block_range_end {
                if i % increment != 0 {
                    continue;
                }
                let fetched_block = full_header_and_proof_result.0.get(&i).unwrap().clone();
                let acc_and_storage = storages_and_proofs_result.get(&i).unwrap().clone();

                headers.push(Header {
                    rlp: fetched_block.0,
                    proof: HeaderProof {
                        leaf_idx: fetched_block.2,
                        mmr_path: fetched_block.1,
                    },
                });

                account_proofs.push(MPTProof {
                    block_number: i,
                    proof: acc_and_storage.1,
                });

                storage_proofs.push(MPTProof {
                    block_number: i,
                    proof: acc_and_storage.3,
                });

                aggregation_set.push(acc_and_storage.2);
            }
            let slot_bytes = Vec::from_hex(slot).expect("Invalid hex string");
            let storage_key = keccak256(slot_bytes).to_string();

            let address_bytes = Vec::from_hex(address).expect("Invalid hex string");
            let account_key = keccak256(address_bytes);

            storages.push(Storage {
                address: address.to_string(),
                slot: slot.to_string(),
                storage_key,
                proofs: storage_proofs,
            });
            accounts.push(Account {
                address: address.to_string(),
                account_key: account_key.to_string(),
                proofs: account_proofs,
            });
        }
        _ => bail!("Unknown collection type"),
    }

    Ok(DatalakeResult {
        compiled_results: aggregation_set,
        headers,
        accounts,
        storages,
        mmr_meta,
    })
}
