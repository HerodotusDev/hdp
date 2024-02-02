use anyhow::Result;
use std::fmt;

/// DataCompiler is a function that returns a vector of DataPoints
type DataCompiler = dyn Fn() -> Result<Vec<DataPoint>>;

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
    pub compilation_pipeline: Vec<Box<DataCompiler>>,
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
        F: Fn() -> Result<Vec<DataPoint>> + 'static,
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

    pub fn compile(&mut self) -> Vec<DataPoint> {
        self.datapoints.clear();
        for compiler in &self.compilation_pipeline {
            self.datapoints.extend(compiler().unwrap());
        }
        self.datapoints.clone()
    }
}

pub trait Derivable {
    fn derive(&self) -> DatalakeBase;
}
