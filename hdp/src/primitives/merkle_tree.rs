use alloy::{
    dyn_abi::DynSolValue,
    primitives::{Keccak256, B256, U256},
};
use alloy_merkle_tree::standard_binary_tree::StandardMerkleTree;

fn raw_result_to_result_commitment(task_commitment: &B256, compiled_result: &U256) -> B256 {
    let mut hasher = Keccak256::new();
    hasher.update(task_commitment);
    hasher.update(compiled_result.to_be_bytes_vec());
    hasher.finalize()
}

/// Build result merkle tree by providing task commitments and task results as reference,
/// And returning tree structure and result commitment
pub fn build_result_merkle_tree(
    tasks_commitments: &[B256],
    task_results: &[U256],
) -> (StandardMerkleTree, Vec<B256>) {
    if tasks_commitments.len() != task_results.len() {
        panic!("tasks commitments and task results have to be same length to construct result merkle tree")
    }
    let mut results_leaves = Vec::new();
    let mut results_commitments = Vec::new();
    for (task_commitment, task_result) in tasks_commitments.iter().zip(task_results.iter()) {
        let result_commitment = raw_result_to_result_commitment(task_commitment, task_result);
        results_commitments.push(result_commitment);
        results_leaves.push(DynSolValue::FixedBytes(result_commitment, 32));
    }
    let tree = StandardMerkleTree::of(&results_leaves);
    (tree, results_commitments)
}

/// Build task merkle tree by providing tasks_commitments as reference
pub fn build_task_merkle_tree(tasks_commitments: &[B256]) -> StandardMerkleTree {
    let mut task_leaves = Vec::new();
    tasks_commitments
        .iter()
        .for_each(|tc| task_leaves.push(DynSolValue::FixedBytes(*tc, 32)));
    StandardMerkleTree::of(&task_leaves)
}

#[cfg(test)]
mod tests {
    use alloy::primitives::b256;

    use super::*;

    #[test]
    fn test_build_result_merkle_tree() {
        let tasks_commitments = vec![B256::ZERO];
        let task_results = vec![U256::from(10)];
        let (tree, results_commitments) =
            build_result_merkle_tree(&tasks_commitments, &task_results);
        let result_root = tree.root();
        assert_eq!(
            result_root,
            b256!("deddf91dc7d95dba2b7698201b4571eaa5bfec0a9ff4276e836f98e3a40a77e9")
        );
        assert_eq!(
            results_commitments,
            vec![b256!(
                "13da86008ba1c6922daee3e07db95305ef49ebced9f5467a0b8613fcc6b343e3"
            )]
        )
    }
}
