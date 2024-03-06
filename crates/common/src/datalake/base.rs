use anyhow::{bail, Ok, Result};
use serde::{Deserialize, Serialize};
use std::{fmt, sync::Arc};
use tokio::sync::RwLock;

use crate::{
    fetcher::AbstractFetcher,
    types::{Account, Header, MMRMeta, Storage},
};

use super::Datalake;

//==============================================================================
// format for input.json
// 1 task = batched blocks
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatalakeResult {
    pub compiled_results: Vec<String>,
    pub headers: Vec<Header>,
    pub accounts: Vec<Account>,
    pub storages: Vec<Storage>,
    pub mmr_meta: MMRMeta,
}

//==============================================================================

/// DatalakeBase is a type that can be used to store data
pub struct DatalakeBase {
    /// Datalake commitment. It is used to identify the datalake
    pub commitment: String,
    pub datalakes_pipeline: Option<Datalake>,
    pub datapoints: Option<DatalakeResult>,
}

impl fmt::Debug for DatalakeBase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DatalakeBase")
            .field("commitment", &self.commitment)
            .field("datalakes_pipeline", &self.datalakes_pipeline)
            .field("datapoints", &self.datapoints)
            .finish()
    }
}

impl DatalakeBase {
    pub fn new(commitment: &str, datalake_type: Datalake) -> Self {
        Self {
            commitment: commitment.to_string(),
            datalakes_pipeline: Some(datalake_type),
            datapoints: None,
        }
    }

    pub async fn compile(
        &mut self,
        fetcher: Arc<RwLock<AbstractFetcher>>,
    ) -> Result<DatalakeResult> {
        let datalake_type = &self.datalakes_pipeline;
        match datalake_type {
            Some(datalake) => {
                let result_datapoints = match datalake {
                    Datalake::BlockSampled(datalake) => datalake.compile(fetcher.clone()).await?,
                    Datalake::DynamicLayout(_) => bail!("dynamic datalake type doesn't support"),
                    Datalake::Unknown => {
                        bail!("Unknown datalake type");
                    }
                };

                self.datapoints = Some(result_datapoints.clone());
                Ok(result_datapoints)
            }
            None => bail!("Datalake type is not defined"),
        }
    }
}

pub trait Derivable {
    fn derive(&self) -> DatalakeBase;
}
