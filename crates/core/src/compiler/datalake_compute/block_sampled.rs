use hdp_primitives::{
    block::account::Account,
    datalake::{
        block_sampled::{BlockSampledCollection, BlockSampledDatalake},
        DatalakeField,
    },
    processed_types::{
        account::ProcessedAccount, header::ProcessedHeader, mmr::MMRMeta, mpt::ProcessedMPTProof,
        storage::ProcessedStorage,
    },
};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc};

use alloy::primitives::{Bytes, U256};
use anyhow::Result;

use hdp_provider::evm::provider::EvmProvider;
use tokio::sync::RwLock;

/// [`CompiledBlockSampledDatalake`] is a unified structure that contains all the required data to verify the datalake
///
/// Contains compiled results, headers, accounts, storages, and mmr_meta data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledBlockSampledDatalake {
    /// Targeted datalake's compiled results
    pub values: Vec<U256>,
    /// Headers related to the datalake
    pub headers: HashSet<ProcessedHeader>,
    /// Accounts related to the datalake
    pub accounts: HashSet<ProcessedAccount>,
    /// Storages related to the datalake
    pub storages: HashSet<ProcessedStorage>,
    /// MMR meta data related to the headers
    pub mmr_meta: MMRMeta,
}

pub async fn compile_block_sampled_datalake(
    datalake: BlockSampledDatalake,
    provider: &Arc<RwLock<EvmProvider>>,
) -> Result<CompiledBlockSampledDatalake> {
    let abstract_provider = provider.write().await;

    let mut aggregation_set: Vec<U256> = Vec::new();

    let (mmr_meta, headers_proofs) = abstract_provider
        .get_range_of_header_proofs(
            datalake.block_range_start,
            datalake.block_range_end,
            datalake.increment,
        )
        .await?;
    let mmr_meta = MMRMeta::from(mmr_meta);
    let mut headers: HashSet<ProcessedHeader> = HashSet::new();
    let mut accounts: HashSet<ProcessedAccount> = HashSet::new();
    let mut storages: HashSet<ProcessedStorage> = HashSet::new();
    let block_range = (datalake.block_range_start..=datalake.block_range_end)
        .step_by(datalake.increment as usize);

    match datalake.sampled_property {
        BlockSampledCollection::Header(property) => {
            for block in block_range {
                let fetched_block = headers_proofs.get(&block).unwrap();
                let value = property
                    .decode_field_from_rlp(&Bytes::from(fetched_block.rlp_block_header.clone()));
                headers.insert(ProcessedHeader::new(
                    fetched_block.rlp_block_header.clone(),
                    fetched_block.element_index,
                    fetched_block.siblings_hashes.clone(),
                ));
                aggregation_set.push(value);
            }
        }
        BlockSampledCollection::Account(address, property) => {
            let accounts_and_proofs_result = abstract_provider
                .get_range_of_account_proofs(
                    datalake.block_range_start,
                    datalake.block_range_end,
                    datalake.increment,
                    address,
                )
                .await?;

            let mut account_proofs: Vec<ProcessedMPTProof> = vec![];
            // let mut encoded_account = "".to_string();

            for block in block_range {
                let fetched_block = headers_proofs.get(&block).unwrap().clone();
                let account_proof = accounts_and_proofs_result.get(&block).unwrap().clone();
                let account = Account::from(&account_proof).rlp_encode();

                let value = property.decode_field_from_rlp(&account);
                headers.insert(ProcessedHeader::new(
                    fetched_block.rlp_block_header.clone(),
                    fetched_block.element_index,
                    fetched_block.siblings_hashes.clone(),
                ));

                let account_proof = ProcessedMPTProof {
                    block_number: block,
                    proof: account_proof.account_proof,
                };

                account_proofs.push(account_proof);
                aggregation_set.push(value);
            }

            accounts.insert(ProcessedAccount::new(address, account_proofs));
        }
        BlockSampledCollection::Storage(address, slot) => {
            let storages_and_proofs_result = abstract_provider
                .get_range_of_storage_proofs(
                    datalake.block_range_start,
                    datalake.block_range_end,
                    datalake.increment,
                    address,
                    slot,
                )
                .await?;

            let mut storage_proofs: Vec<ProcessedMPTProof> = vec![];
            let mut account_proofs: Vec<ProcessedMPTProof> = vec![];

            for i in block_range {
                let fetched_block = headers_proofs.get(&i).unwrap().clone();
                let storage_proof = storages_and_proofs_result.get(&i).unwrap().clone();

                headers.insert(ProcessedHeader::new(
                    fetched_block.rlp_block_header.clone(),
                    fetched_block.element_index,
                    fetched_block.siblings_hashes.clone(),
                ));

                account_proofs.push(ProcessedMPTProof::new(i, storage_proof.account_proof));

                storage_proofs.push(ProcessedMPTProof::new(
                    i,
                    storage_proof.storage_proof[0].proof.clone(),
                ));

                aggregation_set.push(storage_proof.storage_proof[0].value);
            }

            storages.insert(ProcessedStorage::new(address, slot, storage_proofs));
            accounts.insert(ProcessedAccount::new(address, account_proofs));
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
