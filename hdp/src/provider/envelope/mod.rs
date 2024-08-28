use crate::primitives::task::datalake::{envelope::DatalakeEnvelope, DatalakeCompute};

use self::{
    evm::{
        datalake::{FetchError, FetchedDatalake},
        provider::EvmProvider,
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
            _ => panic!("not supported chain id"),
        }
    }

    pub async fn fetch_datalake_envelope(
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
}
