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
    pub identifier: String,
    pub datalakes_pipeline: Vec<Datalake>,
    pub datapoints: Vec<DatalakeResult>,
}

impl fmt::Debug for DatalakeBase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DatalakeBase")
            .field("identifier", &self.identifier)
            .field("datalakes_pipeline", &"datalakes_pipeline")
            .field("datapoints", &"datapoints")
            .finish()
    }
}

impl DatalakeBase {
    pub fn new(identifier: &str, datalake_type: Datalake) -> Self {
        Self {
            identifier: identifier.to_string(),
            datalakes_pipeline: vec![datalake_type],
            datapoints: Vec::new(),
        }
    }

    // TODO: decide if we want to merge datalakes
    // fn merge(&mut self, other: DatalakeBase) {
    //     self.compilation_pipeline.extend(other.compilation_pipeline);
    //     self.identifier = format!("{}{}", self.identifier, other.identifier);
    // }

    // returns the result of the compilation of the datalake
    pub async fn compile(
        &mut self,
        fetcher: Arc<RwLock<AbstractFetcher>>,
    ) -> Result<DatalakeResult> {
        let datalake_type = self.datalakes_pipeline.first().unwrap();
        let result_datapoints = match datalake_type {
            Datalake::BlockSampled(datalake) => datalake.compile(fetcher.clone()).await?,
            Datalake::DynamicLayout(_) => bail!("dynamic datalake type doesn't support"),
            Datalake::Unknown => {
                bail!("Unknown datalake type");
            }
        };

        Ok(result_datapoints)
    }
}

pub trait Derivable {
    fn derive(&self) -> DatalakeBase;
}
