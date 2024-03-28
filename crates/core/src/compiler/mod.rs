use std::{fmt, sync::Arc};

use anyhow::{bail, Result};
use hdp_primitives::datalake::{
    block_sampled::types::{Account, Header, MMRMeta, Storage},
    envelope::DatalakeEnvelope,
};
use hdp_provider::evm::AbstractProvider;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use self::block_sampled::compile_block_sampled_datalake;

pub mod block_sampled;
pub mod test;

/// [`CompiledDatalake`] is a unified structure that contains all the required data to verify the datalake
///
/// Contains compiled results, headers, accounts, storages, and mmr_meta data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledDatalake {
    /// Targeted datalake's compiled results
    pub values: Vec<String>,
    /// Headers related to the datalake
    pub headers: Vec<Header>,
    /// Accounts related to the datalake
    pub accounts: Vec<Account>,
    /// Storages related to the datalake
    pub storages: Vec<Storage>,
    /// MMR meta data related to the headers
    pub mmr_meta: MMRMeta,
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
    ) -> Result<CompiledDatalake> {
        let result_datapoints = match &self.datalake {
            DatalakeEnvelope::BlockSampled(datalake) => {
                compile_block_sampled_datalake(datalake.clone(), provider).await?
            }
            DatalakeEnvelope::Transactions(_) => {
                bail!("Transactions datalake type doesn't support yet")
            }
        };

        Ok(result_datapoints)
    }
}
