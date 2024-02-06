use anyhow::Result;
use std::{fmt, future::Future, pin::Pin};

/// DataCompiler is a function that returns a vector of DataPoints
type DataCompiler =
    Box<dyn Fn() -> Pin<Box<dyn Future<Output = Result<Vec<DataPoint>>> + Send>> + Send>;

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
    pub compilation_pipeline: Vec<DataCompiler>,
    pub datapoints: Vec<DataPoint>,
}

impl fmt::Debug for DatalakeBase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DatalakeBase")
            .field("identifier", &self.identifier)
            .field("compilation_pipeline", &"DataCompilers")
            .field("datapoints", &self.datapoints)
            .finish()
    }
}

impl DatalakeBase {
    pub fn new<F>(identifier: &str, compiler: F) -> Self
    where
        F: Fn() -> Pin<Box<dyn Future<Output = Result<Vec<DataPoint>>> + Send>> + Send + 'static,
    {
        Self {
            identifier: identifier.to_string(),
            compilation_pipeline: vec![Box::new(compiler)],
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
        for compiler in &self.compilation_pipeline {
            // Await the future returned by the compiler and process its result
            let result = compiler().await?;
            self.datapoints.extend(result);
        }
        Ok(self.datapoints.clone())
    }
}

pub trait Derivable {
    fn derive(&self) -> DatalakeBase;
}
