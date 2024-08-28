use crate::primitives::{
    processed_types::block_proofs::ProcessedBlockProofs,
    task::datalake::{envelope::DatalakeEnvelope, DatalakeCompute},
};

use self::{
    evm::{
        datalake::{FetchError, FetchedDatalake},
        from_keys::CategorizedFetchKeys,
        provider::{EvmProvider, ProviderError},
    },
    starknet::StarknetProvider,
};

use super::config::ProviderConfig;

pub mod evm;
pub mod starknet;

pub enum ProviderEnvelope {
    Evm(EvmProvider),
    Starknet(StarknetProvider),
}

impl ProviderEnvelope {
    pub fn new(config: &ProviderConfig) -> Self {
        match config.chain_id {
            1 | 11155111 => Self::Evm(EvmProvider::new(config)),
            // TODO: change chain_id to string
            _ => panic!("not supported chain id"),
        }
    }

    pub async fn fetch_proofs(
        &self,
        datalake: &DatalakeCompute,
    ) -> Result<FetchedDatalake, FetchError> {
        match self {
            ProviderEnvelope::Evm(provider) => match &datalake.datalake {
                DatalakeEnvelope::BlockSampled(datalake) => {
                    provider.fetch_block_sampled(datalake).await
                }
                DatalakeEnvelope::TransactionsInBlock(datalake) => {
                    provider.fetch_transactions(datalake).await
                }
            },
            ProviderEnvelope::Starknet(_) => todo!(),
        }
    }

    pub async fn fetch_proofs_from_keys(
        &self,
        keys: CategorizedFetchKeys,
    ) -> Result<ProcessedBlockProofs, ProviderError> {
        match self {
            ProviderEnvelope::Evm(provider) => provider.fetch_proofs_from_keys(keys).await,
            ProviderEnvelope::Starknet(_) => todo!(),
        }
    }
}
