use anyhow::{bail, Ok, Result};
use serde::{Deserialize, Serialize};
use std::{fmt, sync::Arc};
use tokio::sync::RwLock;

use hdp_primitives::format::{Account, Header, MMRMeta, Storage};
use hdp_provider::evm::AbstractProvider;

use super::Datalake;

/// Datalake result from compilation process
///
/// It contains compiled_results, headers, accounts, storages, and mmr_meta
///
/// All of these data are required to execute the datalake
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatalakeResult {
    /// Targeted datalake's compiled results
    pub compiled_results: Vec<String>,
    /// Headers required for datalake
    pub headers: Vec<Header>,
    /// Accounts required for datalake
    pub accounts: Vec<Account>,
    /// Storages required for datalake
    pub storages: Vec<Storage>,
    /// MMR meta data that stores headers data
    pub mmr_meta: MMRMeta,
}

/// DatalakeBase is unified datalake structure that contains commitment, datalake type, and result
///
/// It is used to identify the datalake and store the result from compilation process
pub struct DatalakeBase {
    /// Datalake commitment. It is used to identify the datalake
    pub commitment: String,
    /// Datalake type
    pub datalake_type: Option<Datalake>,
    /// Datalake result from compilation process
    pub result: Option<DatalakeResult>,
}

impl fmt::Debug for DatalakeBase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DatalakeBase")
            .field("commitment", &self.commitment)
            .field("datalakes_pipeline", &self.datalake_type)
            .field("result", &self.result)
            .finish()
    }
}

impl DatalakeBase {
    /// initialize DatalakeBase with commitment and datalake type
    pub fn new(commitment: &str, datalake_type: Datalake) -> Self {
        Self {
            commitment: commitment.to_string(),
            datalake_type: Some(datalake_type),
            result: None,
        }
    }

    /// Compile the datalake meaning, fetching relevant headers, accounts, storages, and mmr_meta data.
    ///
    /// Plus, it will combine target datalake's datapoints in compiled_results.
    pub async fn compile(
        &mut self,
        provider: &Arc<RwLock<AbstractProvider>>,
    ) -> Result<DatalakeResult> {
        let datalake_type = &self.datalake_type;
        match datalake_type {
            Some(datalake) => {
                let result_datapoints = match datalake {
                    Datalake::BlockSampled(datalake) => datalake.compile(provider).await?,
                    Datalake::DynamicLayout(_) => {
                        bail!("dynamic datalake type doesn't support yet")
                    }
                    Datalake::Transactions(_) => {
                        bail!("Transactions datalake type doesn't support yet")
                    }
                    Datalake::Unknown => {
                        bail!("Unknown datalake type");
                    }
                };

                self.result = Some(result_datapoints.clone());
                Ok(result_datapoints)
            }
            None => bail!("Datalake type is not defined"),
        }
    }
}

pub trait Derivable {
    fn derive(&self) -> DatalakeBase;
}
