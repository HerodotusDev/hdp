use aggregation_functions::AggregationFunction;
use anyhow::{bail, Result};
use std::{collections::HashMap, str::FromStr, sync::Arc};
use tokio::sync::RwLock;

pub mod aggregation_functions;

use common::{
    datalake::{base::Derivable, block_sampled::serialize_sampled_property, Datalake},
    fetcher::AbstractFetcher,
    task::ComputationalTask,
};

pub struct EvaluationResult {
    pub result: HashMap<String, String>,
    pub headers_proof: HashMap<u64, Vec<String>>,
    pub accounts_proof: HashMap<String, HashMap<u64, Vec<String>>>,
}

impl EvaluationResult {
    pub fn new() -> Self {
        EvaluationResult {
            result: HashMap::new(),
            headers_proof: HashMap::new(),
            accounts_proof: HashMap::new(),
        }
    }
    pub fn merkle_commit(&self) -> String {
        "merkle_commit".to_string()
    }
}

impl Default for EvaluationResult {
    fn default() -> Self {
        EvaluationResult::new()
    }
}

pub async fn evaluator(
    mut compute_expressions: Vec<ComputationalTask>,
    datalake_for_tasks: Option<Vec<Datalake>>,
    fetcher: Arc<RwLock<AbstractFetcher>>,
) -> Result<EvaluationResult> {
    let mut results = EvaluationResult::new();
    // If optional datalake_for_tasks is provided, need to assign the datalake to the corresponding task
    if let Some(datalake) = datalake_for_tasks.clone() {
        for (datalake_idx, datalake) in datalake.iter().enumerate() {
            let task = &mut compute_expressions[datalake_idx];

            match datalake {
                Datalake::BlockSampled(block_datalake) => {
                    task.datalake = Some(block_datalake.derive())
                }
                Datalake::DynamicLayout(dynamic_layout_datalake) => {
                    task.datalake = Some(dynamic_layout_datalake.derive())
                }
                _ => bail!("Unknown datalake type"),
            };
        }
    }

    // Evaulate the compute expressions
    for compute_expression in compute_expressions {
        let computation_task_id = compute_expression.to_string();
        let datapoints = compute_expression
            .datalake
            .unwrap()
            .compile(fetcher.clone())
            .await?;
        let aggregation_fn = AggregationFunction::from_str(&compute_expression.aggregate_fn_id)?;
        let aggregation_fn_ctx = compute_expression.aggregate_fn_ctx;
        let result = aggregation_fn.operation(&datapoints, aggregation_fn_ctx)?;
        results.result.insert(computation_task_id, result);
    }

    // Fetch proofs
    // Fetching from url is already done in the compile function. So, no need to fetch again in memory fetcher.
    for datalake in datalake_for_tasks.unwrap() {
        match datalake {
            Datalake::BlockSampled(block_datalake) => {
                let block_range_start = block_datalake.block_range_start;
                let block_range_end = block_datalake.block_range_end;
                let sampled_property = serialize_sampled_property(&block_datalake.sampled_property);
                // if sampled_property is header, then fetch header proofs
                let mut headers_proof = HashMap::new();
                let mut accounts_proof = HashMap::new();
                //    get header proofs from db fetcher
                // for block_number in block_range_start..block_range_end {
                //     let header_proof = fetcher
                //         .read()
                //         .await
                //         .get_rlp_headers_proof(block_number, block_number + increment)
                //         .await?;
                //     headers_proof.insert(block_number, header_proof);
                // }
                results.headers_proof = headers_proof;
                results.accounts_proof = accounts_proof;
            }
            Datalake::DynamicLayout(dynamic_layout_datalake) => {}
            Datalake::Unknown => {}
        }
    }

    Ok(results)
}
