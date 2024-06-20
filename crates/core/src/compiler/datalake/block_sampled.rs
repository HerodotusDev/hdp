use hdp_primitives::{
    datalake::{
        block_sampled::{BlockSampledCollection, BlockSampledDatalake},
        DatalakeField,
    },
    processed_types::{
        account::ProcessedAccount,
        header::{ProcessedHeader, ProcessedHeaderProof},
        mmr::MMRMeta,
        mpt::ProcessedMPTProof,
        storage::ProcessedStorage,
    },
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use alloy_primitives::keccak256;
use anyhow::Result;

use hdp_provider::evm::AbstractProvider;
use tokio::sync::RwLock;

/// [`CompiledBlockSampledDatalake`] is a unified structure that contains all the required data to verify the datalake
///
/// Contains compiled results, headers, accounts, storages, and mmr_meta data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledBlockSampledDatalake {
    /// Targeted datalake's compiled results
    pub values: Vec<String>,
    /// Headers related to the datalake
    pub headers: Vec<ProcessedHeader>,
    /// Accounts related to the datalake
    pub accounts: Vec<ProcessedAccount>,
    /// Storages related to the datalake
    pub storages: Vec<ProcessedStorage>,
    /// MMR meta data related to the headers
    pub mmr_meta: MMRMeta,
}

pub async fn compile_block_sampled_datalake(
    datalake: BlockSampledDatalake,
    provider: &Arc<RwLock<AbstractProvider>>,
) -> Result<CompiledBlockSampledDatalake> {
    let mut abstract_provider = provider.write().await;

    let mut aggregation_set: Vec<String> = Vec::new();

    let full_header_and_proof_result = abstract_provider
        .get_sequencial_full_header_with_proof(datalake.block_range_start, datalake.block_range_end)
        .await?;
    let mmr_meta = full_header_and_proof_result.1;
    let mut headers: Vec<ProcessedHeader> = vec![];
    let mut accounts: Vec<ProcessedAccount> = vec![];
    let mut storages: Vec<ProcessedStorage> = vec![];
    let block_range = (datalake.block_range_start..=datalake.block_range_end)
        .step_by(datalake.increment as usize);

    match datalake.sampled_property {
        BlockSampledCollection::Header(property) => {
            for block in block_range {
                let fetched_block = full_header_and_proof_result.0.get(&block).unwrap().clone();
                let value = property.decode_field_from_rlp(&fetched_block.0);

                headers.push(ProcessedHeader {
                    rlp: fetched_block.0,
                    proof: ProcessedHeaderProof {
                        leaf_idx: fetched_block.2,
                        mmr_path: fetched_block.1,
                    },
                });

                aggregation_set.push(value);
            }
        }
        BlockSampledCollection::Account(address, property) => {
            let accounts_and_proofs_result = abstract_provider
                .get_range_account_with_proof(
                    datalake.block_range_start,
                    datalake.block_range_end,
                    datalake.increment,
                    address.to_string(),
                )
                .await?;

            let mut account_proofs: Vec<ProcessedMPTProof> = vec![];
            // let mut encoded_account = "".to_string();

            for block in block_range {
                let fetched_block = full_header_and_proof_result.0.get(&block).unwrap().clone();
                let account_proof = accounts_and_proofs_result.get(&block).unwrap().clone();
                let value = property.decode_field_from_rlp(&account_proof.encoded_account);

                headers.push(ProcessedHeader {
                    rlp: fetched_block.0,
                    proof: ProcessedHeaderProof {
                        leaf_idx: fetched_block.2,
                        mmr_path: fetched_block.1,
                    },
                });

                let account_proof = ProcessedMPTProof {
                    block_number: block,
                    proof: account_proof.account_proof,
                };

                account_proofs.push(account_proof);
                aggregation_set.push(value);
            }

            let account_key = keccak256(address);
            accounts.push(ProcessedAccount {
                address: address.to_string(),
                account_key: account_key.to_string(),
                proofs: account_proofs,
            });
        }
        BlockSampledCollection::Storage(address, slot) => {
            let storages_and_proofs_result = abstract_provider
                .get_range_storage_with_proof(
                    datalake.block_range_start,
                    datalake.block_range_end,
                    datalake.increment,
                    address.to_string(),
                    slot.to_string(),
                )
                .await?;

            let mut storage_proofs: Vec<ProcessedMPTProof> = vec![];
            let mut account_proofs: Vec<ProcessedMPTProof> = vec![];

            for i in block_range {
                let fetched_block = full_header_and_proof_result.0.get(&i).unwrap().clone();
                let storage_proof = storages_and_proofs_result.get(&i).unwrap().clone();

                headers.push(ProcessedHeader {
                    rlp: fetched_block.0,
                    proof: ProcessedHeaderProof {
                        leaf_idx: fetched_block.2,
                        mmr_path: fetched_block.1,
                    },
                });

                account_proofs.push(ProcessedMPTProof {
                    block_number: i,
                    proof: storage_proof.account_proof,
                });

                storage_proofs.push(ProcessedMPTProof {
                    block_number: i,
                    proof: storage_proof.storage_proof,
                });

                aggregation_set.push(storage_proof.storage_value);
            }

            let storage_key = keccak256(slot).to_string();
            let account_key = keccak256(address);

            storages.push(ProcessedStorage {
                address: address.to_string(),
                slot: slot.to_string(),
                storage_key,
                proofs: storage_proofs,
            });
            accounts.push(ProcessedAccount {
                address: address.to_string(),
                account_key: account_key.to_string(),
                proofs: account_proofs,
            });
        }
    }

    Ok(CompiledBlockSampledDatalake {
        values: aggregation_set,
        headers,
        accounts,
        storages,
        mmr_meta,
    })
}
