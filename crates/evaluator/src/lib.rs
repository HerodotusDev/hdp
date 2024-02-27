use aggregation_functions::AggregationFunction;
use alloy_merkle_tree::standard_binary_tree::StandardMerkleTree;
use alloy_primitives::{hex::FromHex, Keccak256, B256, U256};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    sync::Arc,
};
use tokio::sync::RwLock;

pub mod aggregation_functions;

use common::{
    datalake::{
        base::{DatalakeResult, Derivable},
        Datalake,
    },
    fetcher::AbstractFetcher,
    task::ComputationalTask,
    types::{Account, Header, MMRMeta, ProcessedResult, Storage, Task},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct EvaluationResult {
    /// task_id -> fetched datalake relevant data
    pub fetched_data: HashMap<String, DatalakeResult>,
    /// task_id -> compute result
    pub result: HashMap<String, String>,
    /// ordered task_id
    pub ordered_tasks: Vec<String>,
}

impl EvaluationResult {
    pub fn new() -> Self {
        EvaluationResult {
            result: HashMap::new(),
            ordered_tasks: Vec::new(),
            fetched_data: HashMap::new(),
        }
    }
    pub fn build_merkle_tree(&self) -> (StandardMerkleTree, StandardMerkleTree) {
        let mut tasks_leaves = Vec::new();
        let mut results_leaves = Vec::new();

        for task_id in &self.ordered_tasks {
            let result = self.result.get(task_id).unwrap();
            tasks_leaves.push(task_id.to_string());
            let result = U256::from_str(result).unwrap();
            let mut result_keccak = Keccak256::new();
            let task_id_hex = Vec::from_hex(task_id).unwrap();
            result_keccak.update(task_id_hex);
            result_keccak.update(B256::from(result));
            println!("{:?}", B256::from(result));
            let result_hash = result_keccak.finalize();
            results_leaves.push(result_hash.to_string());
        }
        let tasks_merkle_tree = StandardMerkleTree::of(tasks_leaves);
        let results_merkle_tree = StandardMerkleTree::of(results_leaves);

        (tasks_merkle_tree, results_merkle_tree)
    }

    pub fn save_to_file(&self, file_path: &str) -> Result<()> {
        let file = std::fs::File::create(file_path)?;
        serde_json::to_writer(file, self)?;
        Ok(())
    }

    pub fn to_json(&self) -> Result<String> {
        // 1. build merkle tree
        let (tasks_merkle_tree, results_merkle_tree) = self.build_merkle_tree();
        // 2. get roots
        let task_merkle_root = tasks_merkle_tree.root();
        let result_merkle_root = results_merkle_tree.root();

        // 3. flatten the datalake result for all tasks
        let mut flattened_deaders: HashSet<Header> = HashSet::new();
        let mut flattened_accounts: HashSet<Account> = HashSet::new();
        let mut flattened_storages: HashSet<Storage> = HashSet::new();
        let mut assume_mmr_meta: Option<MMRMeta> = None;

        let mut procesed_tasks: Vec<Task> = vec![];

        for task_id in &self.ordered_tasks {
            let datalake_result = self.fetched_data.get(task_id).unwrap();
            let header_set: HashSet<Header> = datalake_result.headers.iter().cloned().collect();
            let account_set: HashSet<Account> = datalake_result.accounts.iter().cloned().collect();
            let storage_set: HashSet<Storage> = datalake_result.storages.iter().cloned().collect();
            flattened_deaders.extend(header_set);
            flattened_accounts.extend(account_set);
            flattened_storages.extend(storage_set);
            assume_mmr_meta = Some(datalake_result.mmr_meta.clone());

            let result = self.result.get(task_id).unwrap();
            let task_proof = tasks_merkle_tree.get_proof(task_id);
            let result_leaf = evaluation_result_to_leaf(task_id, result);
            let result_proof = results_merkle_tree.get_proof(&result_leaf);

            procesed_tasks.push(Task {
                // TODO: datalake and task serialized bytes
                computational_task: "".to_string(),
                task_commitment: task_id.to_string(),
                task_proof,
                result: result.to_string(),
                result_proof,
                datalake: "".to_string(),
                // TODO: datalake type and property
                datalake_type: 0,
                property: vec![],
            });
        }

        let processed_result = ProcessedResult {
            results_root: result_merkle_root.to_string(),
            tasks_root: task_merkle_root.to_string(),
            headers: flattened_deaders.into_iter().collect(),
            accounts: flattened_accounts.into_iter().collect(),
            mmr: assume_mmr_meta.unwrap(),
            storages: flattened_storages.into_iter().collect(),
            tasks: procesed_tasks,
        };

        Ok(serde_json::to_string(&processed_result)?)
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

        let datalake_result = compute_expression
            .datalake
            .unwrap()
            .compile(fetcher.clone())
            .await?;
        let aggregation_fn = AggregationFunction::from_str(&compute_expression.aggregate_fn_id)?;
        let aggregation_fn_ctx = compute_expression.aggregate_fn_ctx;
        let result =
            aggregation_fn.operation(&datalake_result.compiled_results, aggregation_fn_ctx)?;
        results.result.insert(computation_task_id.clone(), result);
        results.ordered_tasks.push(computation_task_id.clone());
        results
            .fetched_data
            .insert(computation_task_id, datalake_result);
    }

    Ok(results)
}
