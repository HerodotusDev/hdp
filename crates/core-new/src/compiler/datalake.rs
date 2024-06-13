//! Datalake compiler
//! The datalake compiler is responsible for compiling the datalake into a set of fetch keys.
//! The fetch keys are used to fetch the data from the provider.

use std::collections::HashSet;

use hdp_primitives::datalake::{
    block_sampled::BlockSampledCollection, envelope::DatalakeEnvelope,
    transactions::TransactionsCollection,
};
use hdp_provider::key::{
    AccountProviderKey, FetchKeyEnvelope, HeaderProviderKey, StorageProviderKey, TxProviderKey,
    TxReceiptProviderKey,
};

pub struct DatalakeCompiler {}

impl Default for DatalakeCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl DatalakeCompiler {
    pub fn new() -> Self {
        Self {}
    }

    // TODO: depends on the requested field, need to decide whether if this fetch key is able to included or not
    // TODO2: we don't need account key if there is same storage key exists
    pub fn compile(
        &self,
        datalakes: Vec<DatalakeEnvelope>,
        chain_id: u64,
    ) -> HashSet<FetchKeyEnvelope> {
        let mut fetch_set: HashSet<FetchKeyEnvelope> = HashSet::new();
        for datalake in datalakes {
            match datalake {
                DatalakeEnvelope::BlockSampled(datalake) => {
                    let target_blocks: Vec<u64> = (datalake.block_range_start
                        ..datalake.block_range_end)
                        .step_by(datalake.increment as usize)
                        .collect();
                    match datalake.sampled_property {
                        BlockSampledCollection::Header(_) => {
                            for block in target_blocks {
                                fetch_set.insert(FetchKeyEnvelope::Header(HeaderProviderKey::new(
                                    chain_id, block,
                                )));
                            }
                        }
                        BlockSampledCollection::Account(address, _) => {
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
                }
                DatalakeEnvelope::Transactions(datalake) => {
                    let target_tx_index: Vec<u64> = (datalake.start_index..datalake.end_index)
                        .step_by(datalake.increment as usize)
                        .collect();
                    match datalake.sampled_property {
                        TransactionsCollection::Transactions(_) => {
                            for tx_index in target_tx_index {
                                fetch_set.insert(FetchKeyEnvelope::Header(HeaderProviderKey::new(
                                    chain_id,
                                    datalake.target_block,
                                )));
                                fetch_set.insert(FetchKeyEnvelope::Tx(TxProviderKey::new(
                                    chain_id,
                                    datalake.target_block,
                                    tx_index,
                                )));
                            }
                        }
                        TransactionsCollection::TranasactionReceipts(_) => {
                            for tx_index in target_tx_index {
                                fetch_set.insert(FetchKeyEnvelope::Header(HeaderProviderKey::new(
                                    chain_id,
                                    datalake.target_block,
                                )));
                                fetch_set.insert(FetchKeyEnvelope::TxReceipt(
                                    TxReceiptProviderKey::new(
                                        chain_id,
                                        datalake.target_block,
                                        tx_index,
                                    ),
                                ));
                            }
                        }
                    }
                }
            }
        }
        fetch_set
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hdp_primitives::datalake::block_sampled::{BlockSampledDatalake, HeaderField};
    use hdp_primitives::datalake::transactions::{
        IncludedTypes, TransactionField, TransactionsInBlockDatalake,
    };

    #[test]
    fn test_compile() {
        let compiler = DatalakeCompiler::new();
        let datalakes = vec![
            DatalakeEnvelope::BlockSampled(BlockSampledDatalake {
                block_range_start: 0,
                block_range_end: 10,
                increment: 1,
                sampled_property: BlockSampledCollection::Header(HeaderField::BlobGasUsed),
            }),
            DatalakeEnvelope::Transactions(TransactionsInBlockDatalake {
                start_index: 0,
                end_index: 10,
                increment: 1,
                target_block: 0,
                sampled_property: TransactionsCollection::Transactions(TransactionField::GasLimit),
                included_types: IncludedTypes::from(&[1, 1, 1, 1]),
            }),
        ];

        let fetch_keys = compiler.compile(datalakes, 1);
        assert_eq!(fetch_keys.len(), 20);
        assert!(fetch_keys.contains(&FetchKeyEnvelope::Header(HeaderProviderKey::new(1, 0))));
    }
}