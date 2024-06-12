//! Datalake compiler
//! The datalake compiler is responsible for compiling the datalake into a set of fetch keys.
//! The fetch keys are used to fetch the data from the provider.

use std::collections::{HashMap, HashSet};

use hdp_primitives::datalake::{block_sampled::BlockSampledCollection, envelope::DatalakeEnvelope};
use hdp_provider::key::{
    AccountProviderKey, FetchKeyEnvelope, HeaderProviderKey, StorageProviderKey,
};

use super::CompilerResult;

pub struct DatalakeCompiler {}

impl DatalakeCompiler {
    pub fn new() -> Self {
        Self {}
    }

    // TODO: chain_id
    pub fn compile(&self, datalakes: Vec<DatalakeEnvelope>, chain_id: u64) -> CompilerResult {
        let mut chain_map: CompilerResult = HashMap::new();
        let mut fetch_set: HashSet<FetchKeyEnvelope> = HashSet::new();
        for datalake in datalakes {
            match datalake {
                DatalakeEnvelope::BlockSampled(block_sampled) => {
                    //generate target block list startblock - endblock and use increment
                    let target_blocks: Vec<u64> = (block_sampled.block_range_start
                        ..block_sampled.block_range_end)
                        .step_by(block_sampled.increment as usize)
                        .collect();
                    match block_sampled.sampled_property {
                        BlockSampledCollection::Header(field) => {
                            for block in target_blocks {
                                fetch_set.insert(FetchKeyEnvelope::Header(HeaderProviderKey::new(
                                    chain_id, block,
                                )));
                            }
                        }
                        BlockSampledCollection::Account(address, field) => {
                            for block in target_blocks {
                                fetch_set.insert(FetchKeyEnvelope::Header(HeaderProviderKey::new(
                                    chain_id, block,
                                )));
                                fetch_set.insert(FetchKeyEnvelope::Account(
                                    AccountProviderKey::new(chain_id, block, address),
                                ));
                            }
                        }
                        BlockSampledCollection::Storage(address, key) => {
                            for block in target_blocks {
                                fetch_set.insert(FetchKeyEnvelope::Header(HeaderProviderKey::new(
                                    chain_id, block,
                                )));
                                fetch_set.insert(FetchKeyEnvelope::Account(
                                    AccountProviderKey::new(chain_id, block, address),
                                ));
                                fetch_set.insert(FetchKeyEnvelope::Storage(
                                    StorageProviderKey::new(chain_id, block, address, key),
                                ));
                            }
                        }
                    }
                    println!("Compiling block sampled datalake...")
                }
                DatalakeEnvelope::Transactions(tx) => println!("Compiling block datalake..."),
            }
            println!("Compiling datalake...")
        }

        chain_map.insert(chain_id, fetch_set);
        chain_map
    }
}
