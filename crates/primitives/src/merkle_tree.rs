use alloy::{
    dyn_abi::DynSolValue,
    primitives::{Keccak256, B256, U256},
};
use alloy_merkle_tree::standard_binary_tree::StandardMerkleTree;

fn raw_result_to_result_commitment(task_commitment: &B256, compiled_result: &U256) -> B256 {
    let mut hasher = Keccak256::new();
    hasher.update(task_commitment);
    hasher.update(B256::from(*compiled_result));
    hasher.finalize()
}

/// Build result merkle tree by providing task commitments and task results as reference,
/// And returning tree structure and result commitment
pub fn build_result_merkle_tree(
    tasks_commitments: &[B256],
    task_results: &[U256],
) -> (StandardMerkleTree, Vec<B256>) {
    let mut results_leaves = Vec::new();
    let mut results_commitments = Vec::new();
    for (task_commitment, task_result) in tasks_commitments.iter().zip(task_results.iter()) {
        let result_commitment = raw_result_to_result_commitment(task_commitment, task_result);
        results_commitments.push(result_commitment);
        results_leaves.push(DynSolValue::FixedBytes(result_commitment, 32));
    }
    let tree = StandardMerkleTree::of(results_leaves);
    (tree, results_commitments)
}

/// Build task merkle tree by providing tasks_commitments as reference
pub fn build_task_merkle_tree(tasks_commitments: &[B256]) -> StandardMerkleTree {
    let mut task_leaves = Vec::new();
    tasks_commitments
        .iter()
        .for_each(|tc| task_leaves.push(DynSolValue::FixedBytes(*tc, 32)));
    StandardMerkleTree::of(task_leaves)
}
