use hdp_provider::key::FetchKeyEnvelope;
use std::collections::{HashMap, HashSet};

pub mod datalake;
pub mod module;

type CompilerResult = HashMap<u64, HashSet<FetchKeyEnvelope>>;
