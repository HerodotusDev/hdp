use aggregation_functions::AggregationFunction;
use alloy_merkle_tree::standard_binary_tree::StandardMerkleTree;
use alloy_primitives::{hex::FromHex, Keccak256, B256, U256};
use anyhow::{bail, Result};
use std::{collections::HashMap, str::FromStr, sync::Arc};
use tokio::sync::RwLock;

pub mod aggregation_functions;

use common::{
    datalake::{base::Derivable, Datalake},
    fetcher::AbstractFetcher,
    task::ComputationalTask,
};

pub struct EvaluationResult {
    pub result: HashMap<String, String>,
}

impl EvaluationResult {
    pub fn new() -> Self {
        EvaluationResult {
            result: HashMap::new(),
        }
    }
    pub fn merkle_commit(&self) -> (StandardMerkleTree, StandardMerkleTree) {
        let mut tasks_leaves = Vec::new();
        let mut results_leaves = Vec::new();
        for (task_id, result) in self.result.iter() {
            tasks_leaves.push(task_id.to_string());

            let result = U256::from_str(result).unwrap();
            let mut result_keccak = Keccak256::new();
            let task_id_hex = Vec::from_hex(task_id).unwrap();
            result_keccak.update(task_id_hex);
            result_keccak.update(B256::from(result));
            let result_hash = result_keccak.finalize();
            results_leaves.push(result_hash.to_string());
        }
        let tasks_merkle_tree = StandardMerkleTree::of(tasks_leaves);
        let results_merkle_tree = StandardMerkleTree::of(results_leaves);

        (tasks_merkle_tree, results_merkle_tree)
    }
}

pub fn evaluation_result_to_leaf(task_id: &str, result: &str) -> String {
    let result = U256::from_str(result).unwrap();
    let mut result_keccak = Keccak256::new();
    let task_id_hex = Vec::from_hex(task_id).unwrap();
    result_keccak.update(task_id_hex);
    result_keccak.update(B256::from(result));
    let result_hash = result_keccak.finalize();
    result_hash.to_string()
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
    if let Some(datalake) = datalake_for_tasks {
        for (datalake_idx, datalake) in datalake.iter().enumerate() {
            let task = &mut compute_expressions[datalake_idx];

            task.datalake = match datalake {
                Datalake::BlockSampled(block_datalake) => Some(block_datalake.derive()),
                Datalake::DynamicLayout(dynamic_layout_datalake) => {
                    Some(dynamic_layout_datalake.derive())
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

    Ok(results)
}
