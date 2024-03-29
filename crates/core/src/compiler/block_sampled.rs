use hdp_primitives::datalake::{
    block_sampled::{BlockSampledCollection, BlockSampledDatalake},
    DatalakeField,
};
use std::sync::Arc;

use alloy_primitives::keccak256;
use anyhow::Result;

use hdp_primitives::datalake::block_sampled::types::{
    Account, Header, HeaderProof, MPTProof, Storage,
};
use hdp_provider::evm::AbstractProvider;
use tokio::sync::RwLock;

use super::CompiledDatalake;

pub async fn compile_block_sampled_datalake(
    datalake: BlockSampledDatalake,
    provider: &Arc<RwLock<AbstractProvider>>,
) -> Result<CompiledDatalake> {
    let mut abstract_provider = provider.write().await;

    let mut aggregation_set: Vec<String> = Vec::new();

    let full_header_and_proof_result = abstract_provider
        .get_sequencial_full_header_with_proof(datalake.block_range_start, datalake.block_range_end)
        .await?;
    let mmr_meta = full_header_and_proof_result.1;
    let mut headers: Vec<Header> = vec![];
    let mut accounts: Vec<Account> = vec![];
    let mut storages: Vec<Storage> = vec![];

    match datalake.sampled_property {
        BlockSampledCollection::Header(property) => {
            for block in datalake.block_range_start..=datalake.block_range_end {
                if block % datalake.increment != 0 {
                    continue;
                }
                let fetched_block = full_header_and_proof_result.0.get(&block).unwrap().clone();
                let value = property.decode_field_from_rlp(&fetched_block.0);

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
        BlockSampledCollection::Account(address, property) => {
            let accounts_and_proofs_result = abstract_provider
                .get_range_account_with_proof(
                    datalake.block_range_start,
                    datalake.block_range_end,
                    datalake.increment,
                    address.to_string(),
                )
                .await?;

            let mut account_proofs: Vec<MPTProof> = vec![];
            // let mut encoded_account = "".to_string();

            for block in datalake.block_range_start..=datalake.block_range_end {
                if block % datalake.increment != 0 {
                    continue;
                }
                let fetched_block = full_header_and_proof_result.0.get(&block).unwrap().clone();
                let acc = accounts_and_proofs_result.get(&block).unwrap().clone();
                // encoded_account = acc.0.clone();
                let value = property.decode_field_from_rlp(&acc.0);

                headers.push(Header {
                    rlp: fetched_block.0,
                    proof: HeaderProof {
                        leaf_idx: fetched_block.2,
                        mmr_path: fetched_block.1,
                    },
                });

                let account_proof = MPTProof {
                    block_number: block,
                    proof: acc.1,
                };

                account_proofs.push(account_proof);
                aggregation_set.push(value);
            }

            let account_key = keccak256(address);

            accounts.push(Account {
                address: address.to_string(),
                account_key: account_key.to_string(),
                proofs: account_proofs,
            });
        }
        BlockSampledCollection::Storage(address, slot) => {
            println!("Storage :{:?}", slot);

            let storages_and_proofs_result = abstract_provider
                .get_range_storage_with_proof(
                    datalake.block_range_start,
                    datalake.block_range_end,
                    datalake.increment,
                    address.to_string(),
                    slot.to_string(),
                )
                .await?;

            let mut storage_proofs: Vec<MPTProof> = vec![];
            let mut account_proofs: Vec<MPTProof> = vec![];

            for i in datalake.block_range_start..=datalake.block_range_end {
                if i % datalake.increment != 0 {
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

            let storage_key = keccak256(slot).to_string();
            println!("Storage key :{:?}", storage_key);
            println!("address  :{:?}", address);

            let account_key = keccak256(address);

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
    }

    Ok(CompiledDatalake {
        values: aggregation_set,
        headers,
        accounts,
        storages,
        mmr_meta,
    })
}
