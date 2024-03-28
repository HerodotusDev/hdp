use std::{fmt, sync::Arc};

use anyhow::{bail, Result};
use hdp_primitives::{
    datalake::envelope::DatalakeEnvelope,
    format::{Account, Header, MMRMeta, Storage},
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
    pub datalake: Option<DatalakeEnvelope>,
    /// Datalake result from compilation process
    pub result: Option<CompiledDatalake>,
}

impl fmt::Debug for DatalakeCompiler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DatalakeCompiler")
            .field("commitment", &self.commitment)
            .field("datalakes_pipeline", &self.datalake)
            .field("result", &self.result)
            .finish()
    }
}

impl DatalakeCompiler {
    /// initialize DatalakeCompiler with commitment and datalake
    pub fn new(commitment: &str, datalake: DatalakeEnvelope) -> Self {
        Self {
            commitment: commitment.to_string(),
            datalake: Some(datalake),
            result: None,
        }
    }

    /// Compile the datalake meaning, fetching relevant headers, accounts, storages, and mmr_meta data.
    ///
    /// Plus, it will combine target datalake's datapoints in compiled_results.
    pub async fn compile(
        &mut self,
        provider: &Arc<RwLock<AbstractProvider>>,
    ) -> Result<CompiledDatalake> {
        match &self.datalake {
            Some(datalake) => {
                let result_datapoints = match datalake {
                    DatalakeEnvelope::BlockSampled(datalake) => {
                        compile_block_sampled_datalake(datalake.clone(), provider).await?
                    }
                    DatalakeEnvelope::Transactions(_) => {
                        bail!("Transactions datalake type doesn't support yet")
                    }
                };

                self.result = Some(result_datapoints.clone());
                Ok(result_datapoints)
            }
            None => bail!("Datalake type is not defined"),
        }
    }
}

/// Transform different datalake types into DatalakeCompiler
impl Derivable for DatalakeEnvelope {
    fn derive(&self) -> DatalakeCompiler {
        match self {
            DatalakeEnvelope::BlockSampled(datalake) => DatalakeCompiler::new(
                &datalake.commit(),
                DatalakeEnvelope::BlockSampled(datalake.clone()),
            ),
            DatalakeEnvelope::Transactions(datalake) => DatalakeCompiler::new(
                &datalake.commit(),
                DatalakeEnvelope::Transactions(datalake.clone()),
            ),
        }
    }
}

pub trait Derivable {
    fn derive(&self) -> DatalakeCompiler;
}
