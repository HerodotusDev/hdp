use crate::{
    primitives::{
        block::account::Account,
        processed_types::{
            account::ProcessedAccount, block_proofs::convert_to_mmr_with_headers,
            header::ProcessedHeader, mmr::MMRMeta, mpt::ProcessedMPTProof,
            storage::ProcessedStorage,
        },
        task::datalake::{
            block_sampled::{BlockSampledCollection, BlockSampledDatalake},
            DatalakeField,
        },
    },
    provider::{error::ProviderError, evm::provider::EvmProvider, types::FetchedDatalake},
};
use std::collections::{HashMap, HashSet};

use alloy::primitives::{Bytes, U256};
use anyhow::Result;

impl EvmProvider {
    pub(crate) async fn fetch_block_sampled(
        &self,
        datalake: &BlockSampledDatalake,
    ) -> Result<FetchedDatalake, ProviderError> {
        let mut aggregation_set: Vec<U256> = Vec::new();

        let headers_proofs = self
            .get_range_of_header_proofs(
                datalake.block_range_start,
                datalake.block_range_end,
                datalake.increment,
            )
            .await?;
        let mut mmr_with_headers: HashMap<MMRMeta, HashSet<ProcessedHeader>> = HashMap::new();

        let mut accounts: HashSet<ProcessedAccount> = HashSet::new();
        let mut storages: HashSet<ProcessedStorage> = HashSet::new();
        let block_range = (datalake.block_range_start..=datalake.block_range_end)
            .step_by(datalake.increment as usize);

        match &datalake.sampled_property {
            BlockSampledCollection::Header(property) => {
                for block in block_range {
                    let (fetched_block, mmr) = headers_proofs.get(&block).unwrap();
                    let value = property.decode_field_from_rlp(&Bytes::from(
                        fetched_block.rlp_block_header.clone(),
                    ));
                    let processed_header = ProcessedHeader::new(
                        fetched_block.rlp_block_header.clone(),
                        fetched_block.element_index,
                        fetched_block.siblings_hashes.clone(),
                    );
                    aggregation_set.push(value);
                    mmr_with_headers
                        .entry(mmr.clone())
                        .and_modify(|existing_headers| {
                            existing_headers.insert(processed_header.clone());
                        })
                        .or_insert_with(|| {
                            let mut new_set = HashSet::new();
                            new_set.insert(processed_header);
                            new_set
                        });
                }
            }
            BlockSampledCollection::Account(address, property) => {
                let accounts_and_proofs_result = self
                    .get_range_of_account_proofs(
                        datalake.block_range_start,
                        datalake.block_range_end,
                        datalake.increment,
                        *address,
                    )
                    .await?;

                let mut account_proofs: Vec<ProcessedMPTProof> = vec![];

                for block in block_range {
                    let (fetched_block, mmr) = headers_proofs.get(&block).unwrap().clone();
                    let account_proof = accounts_and_proofs_result.get(&block).unwrap().clone();
                    let account = Account::from(&account_proof).rlp_encode();

                    let value = property.decode_field_from_rlp(&account);
                    let processed_header = ProcessedHeader::new(
                        fetched_block.rlp_block_header.clone(),
                        fetched_block.element_index,
                        fetched_block.siblings_hashes.clone(),
                    );

                    let account_proof = ProcessedMPTProof {
                        block_number: block,
                        proof: account_proof.account_proof,
                    };

                    account_proofs.push(account_proof);
                    aggregation_set.push(value);
                    mmr_with_headers
                        .entry(mmr.clone())
                        .and_modify(|existing_headers| {
                            existing_headers.insert(processed_header.clone());
                        })
                        .or_insert_with(|| {
                            let mut new_set = HashSet::new();
                            new_set.insert(processed_header);
                            new_set
                        });
                }

                accounts.insert(ProcessedAccount::new(*address, account_proofs));
            }
            BlockSampledCollection::Storage(address, slot) => {
                let storages_and_proofs_result = self
                    .get_range_of_storage_proofs(
                        datalake.block_range_start,
                        datalake.block_range_end,
                        datalake.increment,
                        *address,
                        *slot,
                    )
                    .await?;

                let mut storage_proofs: Vec<ProcessedMPTProof> = vec![];
                let mut account_proofs: Vec<ProcessedMPTProof> = vec![];

                for i in block_range {
                    let (fetched_block, mmr) = headers_proofs.get(&i).unwrap().clone();
                    let storage_proof = storages_and_proofs_result.get(&i).unwrap().clone();

                    let processed_header = ProcessedHeader::new(
                        fetched_block.rlp_block_header.clone(),
                        fetched_block.element_index,
                        fetched_block.siblings_hashes.clone(),
                    );

                    account_proofs.push(ProcessedMPTProof::new(i, storage_proof.account_proof));

                    storage_proofs.push(ProcessedMPTProof::new(
                        i,
                        storage_proof.storage_proof[0].proof.clone(),
                    ));

                    aggregation_set.push(storage_proof.storage_proof[0].value);
                    mmr_with_headers
                        .entry(mmr.clone())
                        .and_modify(|existing_headers| {
                            existing_headers.insert(processed_header.clone());
                        })
                        .or_insert_with(|| {
                            let mut new_set = HashSet::new();
                            new_set.insert(processed_header);
                            new_set
                        });
                }

                storages.insert(ProcessedStorage::new(*address, *slot, storage_proofs));
                accounts.insert(ProcessedAccount::new(*address, account_proofs));
            }
        }

        Ok(FetchedDatalake {
            values: aggregation_set,
            mmr_with_headers: HashSet::from_iter(convert_to_mmr_with_headers(mmr_with_headers)),
            accounts,
            storages,
            transactions: HashSet::new(),
            transaction_receipts: HashSet::new(),
        })
    }
}
