use aggregation_functions::AggregationFunction;
use alloy_dyn_abi::DynSolValue;
use alloy_merkle_tree::standard_binary_tree::StandardMerkleTree;
use alloy_primitives::{hex::FromHex, FixedBytes, Keccak256, B256, U256};
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
    types::{
        split_big_endian_hex_into_parts, Account, AccountFormatted, Header, HeaderFormatted,
        MMRMeta, ProcessedResult, ProcessedResultFormatted, Storage, StorageFormatted, Task,
        TaskFormatted,
    },
};

#[derive(Serialize, Deserialize, Debug)]
pub struct EvaluationResult {
    /// task_commitment -> fetched datalake relevant data
    pub fetched_data: HashMap<String, DatalakeResult>,
    /// task_commitment -> computed result
    pub result: HashMap<String, String>,
    /// ordered task_commitment
    pub ordered_tasks: Vec<String>,
    /// encoded tasks task_commitment -> encoded task
    pub encoded_tasks: HashMap<String, String>,
    /// encoded datalakes task_commitment -> evaluated datalake
    pub encoded_datalakes: HashMap<String, EvaluatedDatalake>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EvaluatedDatalake {
    /// encoded datalake
    encoded_datalake: String,
    /// ex. dynamic datalake / block sampled datalake
    datalake_type: u8,
    /// ex. "header", "account", "storage"
    property_type: u8,
}

impl EvaluationResult {
    pub fn new() -> Self {
        EvaluationResult {
            result: HashMap::new(),
            ordered_tasks: Vec::new(),
            fetched_data: HashMap::new(),
            encoded_tasks: HashMap::new(),
            encoded_datalakes: HashMap::new(),
        }
    }
    pub fn build_merkle_tree(&self) -> (StandardMerkleTree, StandardMerkleTree) {
        let mut tasks_leaves = Vec::new();
        let mut results_leaves = Vec::new();

        for task_commitment in &self.ordered_tasks {
            let result = self.result.get(task_commitment).unwrap();

            let typed_task_commitment = FixedBytes::from_hex(task_commitment).unwrap();
            tasks_leaves.push(DynSolValue::FixedBytes(typed_task_commitment, 32));

            let result_commitment = evaluation_result_to_result_commitment(task_commitment, result);
            results_leaves.push(DynSolValue::FixedBytes(result_commitment, 32));
        }
        let tasks_merkle_tree = StandardMerkleTree::of(tasks_leaves);
        let results_merkle_tree = StandardMerkleTree::of(results_leaves);

        (tasks_merkle_tree, results_merkle_tree)
    }

    pub fn save_to_file(&self, file_path: &str, is_cairo_format: bool) -> Result<()> {
        let json = if is_cairo_format {
            self.to_cairo_formatted_json()?
        } else {
            self.to_general_json()?
        };
        std::fs::write(file_path, json)?;
        Ok(())
    }

    pub fn to_general_json(&self) -> Result<String> {
        // 1. build merkle tree
        let (tasks_merkle_tree, results_merkle_tree) = self.build_merkle_tree();

        // 2. get roots of merkle tree
        let task_merkle_root = tasks_merkle_tree.root();
        let result_merkle_root = results_merkle_tree.root();

        // 3. flatten the datalake result for all tasks
        let mut flattened_headers: HashSet<Header> = HashSet::new();
        let mut flattened_accounts: HashSet<Account> = HashSet::new();
        let mut flattened_storages: HashSet<Storage> = HashSet::new();
        let mut assume_mmr_meta: Option<MMRMeta> = None;

        let mut procesed_tasks: Vec<Task> = vec![];

        for task_commitment in &self.ordered_tasks {
            let datalake_result = self.fetched_data.get(task_commitment).unwrap();
            let header_set: HashSet<Header> = datalake_result.headers.iter().cloned().collect();
            let account_set: HashSet<Account> = datalake_result.accounts.iter().cloned().collect();
            let storage_set: HashSet<Storage> = datalake_result.storages.iter().cloned().collect();
            flattened_headers.extend(header_set);
            flattened_accounts.extend(account_set);
            flattened_storages.extend(storage_set);
            assume_mmr_meta = Some(datalake_result.mmr_meta.clone());

            let result = self.result.get(task_commitment).unwrap();
            let typed_task_commitment = FixedBytes::from_hex(task_commitment).unwrap();
            let task_proof =
                tasks_merkle_tree.get_proof(&DynSolValue::FixedBytes(typed_task_commitment, 32));
            let result_commitment = evaluation_result_to_result_commitment(task_commitment, result);
            let result_proof =
                results_merkle_tree.get_proof(&DynSolValue::FixedBytes(result_commitment, 32));
            let encoded_task = self.encoded_tasks.get(task_commitment).unwrap().to_string();
            let datalake = self.encoded_datalakes.get(task_commitment).unwrap();

            procesed_tasks.push(Task {
                encoded_task,
                task_commitment: task_commitment.to_string(),
                task_proof,
                result: result.to_string(),
                result_commitment: result_commitment.to_string(),
                result_proof,
                encoded_datalake: datalake.encoded_datalake.clone(),
                datalake_type: datalake.datalake_type,
                property_type: datalake.property_type,
            });
        }

        let processed_result = ProcessedResult {
            results_root: result_merkle_root.to_string(),
            tasks_root: task_merkle_root.to_string(),
            headers: flattened_headers.into_iter().collect(),
            accounts: flattened_accounts.into_iter().collect(),
            mmr: assume_mmr_meta.unwrap(),
            storages: flattened_storages.into_iter().collect(),
            tasks: procesed_tasks,
        };

        Ok(serde_json::to_string(&processed_result)?)
    }

    pub fn to_cairo_formatted_json(&self) -> Result<String> {
        // 1. build merkle tree
        let (tasks_merkle_tree, results_merkle_tree) = self.build_merkle_tree();
        // 2. get roots
        let task_merkle_root = tasks_merkle_tree.root();
        let result_merkle_root = results_merkle_tree.root();

        // 3. flatten the datalake result for all tasks
        let mut flattened_deaders: HashSet<HeaderFormatted> = HashSet::new();
        let mut flattened_accounts: HashSet<AccountFormatted> = HashSet::new();
        let mut flattened_storages: HashSet<StorageFormatted> = HashSet::new();
        let mut assume_mmr_meta: Option<MMRMeta> = None;

        let mut procesed_tasks: Vec<TaskFormatted> = vec![];

        for task_commitment in &self.ordered_tasks {
            let datalake_result = self.fetched_data.get(task_commitment).unwrap();
            let header_set: HashSet<HeaderFormatted> = datalake_result
                .headers
                .iter()
                .cloned()
                .map(|h| h.to_cairo_format())
                .collect();
            let account_set: HashSet<AccountFormatted> = datalake_result
                .accounts
                .iter()
                .cloned()
                .map(|a| a.to_cairo_format())
                .collect();
            let storage_set: HashSet<StorageFormatted> = datalake_result
                .storages
                .iter()
                .cloned()
                .map(|s| s.to_cairo_format())
                .collect();
            flattened_deaders.extend(header_set);
            flattened_accounts.extend(account_set);
            flattened_storages.extend(storage_set);
            assume_mmr_meta = Some(datalake_result.mmr_meta.clone());

            let result = self.result.get(task_commitment).unwrap();
            let typed_task_commitment = FixedBytes::from_hex(task_commitment).unwrap();
            let task_proof =
                tasks_merkle_tree.get_proof(&DynSolValue::FixedBytes(typed_task_commitment, 32));
            let result_commitment = evaluation_result_to_result_commitment(task_commitment, result);
            let result_proof =
                results_merkle_tree.get_proof(&DynSolValue::FixedBytes(result_commitment, 32));
            let encoded_task = self.encoded_tasks.get(task_commitment).unwrap().to_string();
            let evaluated_datalake = self.encoded_datalakes.get(task_commitment).unwrap();
            let task = Task {
                encoded_task,
                task_commitment: task_commitment.to_string(),
                task_proof,
                result: result.to_string(),
                result_commitment: result_commitment.to_string(),
                result_proof,
                encoded_datalake: evaluated_datalake.encoded_datalake.clone(),
                datalake_type: evaluated_datalake.datalake_type,
                property_type: evaluated_datalake.property_type,
            };

            procesed_tasks.push(task.to_cairo_format());
        }

        let processed_result = ProcessedResultFormatted {
            results_root: split_big_endian_hex_into_parts(&result_merkle_root.to_string()),
            tasks_root: split_big_endian_hex_into_parts(&task_merkle_root.to_string()),
            headers: flattened_deaders.into_iter().collect(),
            accounts: flattened_accounts.into_iter().collect(),
            mmr: assume_mmr_meta.unwrap(),
            storages: flattened_storages.into_iter().collect(),
            tasks: procesed_tasks,
        };

        Ok(serde_json::to_string(&processed_result)?)
    }
}

pub fn evaluation_result_to_result_commitment(
    task_commitment: &str,
    result: &str,
) -> FixedBytes<32> {
    let result = U256::from_str(result).unwrap();
    let mut hasher = Keccak256::new();
    let task_commitment_bytes = Vec::from_hex(task_commitment).unwrap();
    hasher.update(task_commitment_bytes);
    hasher.update(B256::from(result));
    hasher.finalize()
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
        let encoded_task = compute_expression.encode()?;
        let mut datalake = compute_expression.datalake.unwrap();
        // TODO: in v0 we consider datalake pipeline is single datalake
        let encoded_datalake = datalake.datalakes_pipeline[0].serialize()?;
        let datalake_result = datalake.compile(fetcher.clone()).await?;
        let aggregation_fn = AggregationFunction::from_str(&compute_expression.aggregate_fn_id)?;
        let aggregation_fn_ctx = &compute_expression.aggregate_fn_ctx;
        let result = aggregation_fn.operation(
            &datalake_result.compiled_results,
            aggregation_fn_ctx.clone(),
        )?;
        results.result.insert(computation_task_id.clone(), result);
        results.ordered_tasks.push(computation_task_id.clone());
        results
            .fetched_data
            .insert(computation_task_id.clone(), datalake_result);
        results
            .encoded_tasks
            .insert(computation_task_id.clone(), encoded_task);
        results.encoded_datalakes.insert(
            computation_task_id,
            EvaluatedDatalake {
                encoded_datalake,
                datalake_type: datalake.datalakes_pipeline[0].get_datalake_type(),
                property_type: datalake.datalakes_pipeline[0].get_property_type(),
            },
        );
    }

    Ok(results)
}
