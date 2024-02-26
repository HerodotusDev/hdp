use anyhow::{bail, Ok, Result};
use std::{fmt, sync::Arc};
use tokio::sync::RwLock;

use crate::fetcher::AbstractFetcher;

use super::Datalake;

//==============================================================================
// format for input.json
// 1 task = batched blocks
#[derive(Debug, Clone, PartialEq)]
pub struct DatalakeResult {
    pub mmr: Vec<MMRResult>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MMRResult {
    pub compiled_result: Vec<String>,
    pub blocks: Vec<BlockResult>,
    pub mmr_meta: MMRMetaResult,
}

// depends on the datalake type
#[derive(Debug, Clone, PartialEq)]
pub struct BlockResult {
    // header data
    pub leaf_idx: u64,
    pub mmr_proof: Vec<String>,
    pub rlp_encoded_header: String,
    // account data
    pub account: Option<AccountResult>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccountResult {
    pub address: String,
    pub account_proof: Vec<String>,
    pub rlp_encoded_account: String,
    pub storage: Option<StorageResult>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StorageResult {
    pub storage_proof: Vec<String>,
    pub storage_key: String,
    pub storage_value: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MMRMetaResult {
    pub mmr_id: u64,
    pub mmr_peaks: Vec<String>,
    pub mmr_root: String,
    pub mmr_size: u64,
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
