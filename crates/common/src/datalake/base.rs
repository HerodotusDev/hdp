use anyhow::{bail, Result};
use std::fmt;

use super::Datalake;

/// DataPoint is a type that can be used to store data in a Datalake
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum DataPoint {
    Int(u64),
    Float(f64),
    Str(String),
}

/// DatalakeBase is a type that can be used to store data
pub struct DatalakeBase {
    pub identifier: String,
    pub datalakes_pipeline: Vec<Datalake>,
    pub datapoints: Vec<DataPoint>,
}

impl fmt::Debug for DatalakeBase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DatalakeBase")
            .field("identifier", &self.identifier)
            .field("datalakes_pipeline", &"datalakes_pipeline")
            .field("datapoints", &self.datapoints)
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

    pub async fn compile(&mut self) -> Result<Vec<DataPoint>> {
        self.datapoints.clear();
        for datalake_type in &self.datalakes_pipeline {
            let result_datapoints = match datalake_type {
                Datalake::BlockSampled(datalake) => datalake.compile().await?,
                Datalake::DynamicLayout(datalake) => datalake.compile().await?,
                Datalake::Unknown => {
                    bail!("Unknown datalake type");
                }
            };
            self.datapoints.extend(result_datapoints);
        }
        Ok(self.datapoints.clone())
    }
}

pub trait Derivable {
    fn derive(&self) -> DatalakeBase;
}
