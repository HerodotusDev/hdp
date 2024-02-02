use aggregation_functions::AggregationFunction;
use anyhow::{bail, Result};
use std::{collections::HashMap, str::FromStr};

pub mod aggregation_functions;

use types::{
    datalake::base::{DataPoint, Derivable},
    task::ComputationalTask,
    Datalake,
};

pub struct EvaluationResult {
    pub result: HashMap<String, DataPoint>,
}

impl EvaluationResult {
    pub fn new() -> Self {
        EvaluationResult {
            result: HashMap::new(),
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

pub fn evaluator(
    mut compute_expressions: Vec<ComputationalTask>,
    datalake_for_tasks: Option<Vec<Datalake>>,
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
        let datapoints = compute_expression.datalake.unwrap().compile();
        let aggregation_fn = AggregationFunction::from_str(&compute_expression.aggregate_fn_id)?;
        let aggregation_fn_ctx = compute_expression.aggregate_fn_ctx;
        let result = aggregation_fn.operation(&datapoints, aggregation_fn_ctx)?;
        results.result.insert(computation_task_id, result);
    }

    Ok(results)
}
