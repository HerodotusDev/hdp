use std::{fmt, sync::Arc};

use anyhow::{bail, Result};
use hdp_primitives::datalake::envelope::DatalakeEnvelope;
use hdp_provider::evm::AbstractProvider;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use self::{
    block_sampled::{compile_block_sampled_datalake, CompiledBlockSampledDatalake},
    transactions::CompiledTransactionsDatalake,
};

pub mod block_sampled;
pub mod test;
pub mod transactions;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CompiledDatalakeEnvelope {
    /// Block sampled datalake
    BlockSampled(CompiledBlockSampledDatalake),
    /// Transactions datalake
    Transactions(CompiledTransactionsDatalake),
}

impl CompiledDatalakeEnvelope {
    ///Get values from compiled datalake
    pub fn get_values(&self) -> Vec<String> {
        match self {
            CompiledDatalakeEnvelope::BlockSampled(compiled_block_sampled_datalake) => {
                compiled_block_sampled_datalake.values.clone()
            }
            CompiledDatalakeEnvelope::Transactions(compiled_transactions_datalake) => {
                compiled_transactions_datalake.values.clone()
            }
        }
    }
}

pub struct DatalakeCompiler {
    /// Datalake commitment. It is used to identify the datalake
    pub commitment: String,
    /// Datalake
    pub datalake: DatalakeEnvelope,
}

impl fmt::Debug for DatalakeCompiler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DatalakeCompiler")
            .field("commitment", &self.commitment)
            .field("datalakes_pipeline", &self.datalake)
            .finish()
    }
}

impl DatalakeCompiler {
    /// initialize DatalakeCompiler with commitment and datalake
    pub fn new(datalake: DatalakeEnvelope) -> Self {
        Self {
            commitment: datalake.get_commitment(),
            datalake,
        }
    }

    /// Compile the datalake meaning, fetching relevant headers, accounts, storages, and mmr_meta data.
    ///
    /// Plus, it will combine target datalake's datapoints in compiled_results.
    pub async fn compile(
        &self,
        provider: &Arc<RwLock<AbstractProvider>>,
    ) -> Result<CompiledDatalakeEnvelope> {
        let result_datapoints = match &self.datalake {
            DatalakeEnvelope::BlockSampled(datalake) => CompiledDatalakeEnvelope::BlockSampled(
                compile_block_sampled_datalake(datalake.clone(), provider).await?,
            ),
            DatalakeEnvelope::Transactions(_) => {
                bail!("Transactions datalake type doesn't support yet")
            }
        };

        Ok(result_datapoints)
    }
}
