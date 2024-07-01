use hdp_primitives::{
    block::account::Account,
    processed_types::{
        account::ProcessedAccount, header::ProcessedHeader, mmr::MMRMeta, mpt::ProcessedMPTProof,
        storage::ProcessedStorage,
    },
    task::datalake::{
        block_sampled::{BlockSampledCollection, BlockSampledDatalake},
        DatalakeField,
    },
};
use std::collections::HashSet;

use alloy::primitives::{Bytes, U256};
use anyhow::Result;

use hdp_provider::evm::provider::EvmProvider;

use super::{FetchError, Fetchable, FetchedDatalake};

impl Fetchable for BlockSampledDatalake {
    async fn fetch(&self, provider: EvmProvider) -> Result<FetchedDatalake, FetchError> {
        let mut aggregation_set: Vec<U256> = Vec::new();

        let (mmr_meta, headers_proofs) = provider
            .get_range_of_header_proofs(
                self.block_range_start,
                self.block_range_end,
                self.increment,
            )
            .await?;
        let mmr_meta = MMRMeta::from(mmr_meta);
        let mut headers: HashSet<ProcessedHeader> = HashSet::new();
        let mut accounts: HashSet<ProcessedAccount> = HashSet::new();
        let mut storages: HashSet<ProcessedStorage> = HashSet::new();
        let block_range =
            (self.block_range_start..=self.block_range_end).step_by(self.increment as usize);

        match &self.sampled_property {
            BlockSampledCollection::Header(property) => {
                for block in block_range {
                    let fetched_block = headers_proofs.get(&block).unwrap();
                    let value = property.decode_field_from_rlp(&Bytes::from(
                        fetched_block.rlp_block_header.clone(),
                    ));
                    headers.insert(ProcessedHeader::new(
                        fetched_block.rlp_block_header.clone(),
                        fetched_block.element_index,
                        fetched_block.siblings_hashes.clone(),
                    ));
                    aggregation_set.push(value);
                }
            }
            BlockSampledCollection::Account(address, property) => {
                let accounts_and_proofs_result = provider
                    .get_range_of_account_proofs(
                        self.block_range_start,
                        self.block_range_end,
                        self.increment,
                        *address,
                    )
                    .await?;

                let mut account_proofs: Vec<ProcessedMPTProof> = vec![];

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

                accounts.insert(ProcessedAccount::new(*address, account_proofs));
            }
            BlockSampledCollection::Storage(address, slot) => {
                let storages_and_proofs_result = provider
                    .get_range_of_storage_proofs(
                        self.block_range_start,
                        self.block_range_end,
                        self.increment,
                        *address,
                        *slot,
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

                storages.insert(ProcessedStorage::new(*address, *slot, storage_proofs));
                accounts.insert(ProcessedAccount::new(*address, account_proofs));
            }
        }

        Ok(FetchedDatalake {
            values: aggregation_set,
            headers,
            accounts,
            storages,
            transactions: HashSet::new(),
            transaction_receipts: HashSet::new(),
            mmr_meta,
        })
    }
}
