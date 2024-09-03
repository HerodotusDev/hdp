use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use starknet_types_core::{felt::Felt, hash::StarkHash};

/// Codebase is from https://github.com/eqlabs/pathfinder/tree/ae81d84b7c4157891069bd02ef810a29b60a94e3

/// Holds the membership/non-membership of a contract and its associated
/// contract contract if the contract exists.
#[derive(Debug, Serialize, Deserialize)]
#[skip_serializing_none]
pub struct GetProofOutput {
    /// The global state commitment for Starknet 0.11.0 blocks onwards, if
    /// absent the hash of the first node in the
    /// [contract_proof](GetProofOutput#contract_proof) is the global state
    /// commitment.
    state_commitment: Option<Felt>,
    /// Required to verify that the hash of the class commitment and the root of
    /// the [contract_proof](GetProofOutput::contract_proof) matches the
    /// [state_commitment](Self#state_commitment). Present only for Starknet
    /// blocks 0.11.0 onwards.
    class_commitment: Option<Felt>,

    /// Membership / Non-membership proof for the queried contract
    contract_proof: Vec<TrieNode>,

    /// Additional contract data if it exists.
    contract_data: Option<ContractData>,
}

/// A node in a Starknet patricia-merkle trie.
///
/// See pathfinders merkle-tree crate for more information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TrieNode {
    Binary { left: Felt, right: Felt },
    Edge { child: Felt, path: [u8; 32] },
}

impl TrieNode {
    pub fn hash<H: StarkHash>(&self) -> Felt {
        match self {
            TrieNode::Binary { left, right } => H::hash(left, right),
            TrieNode::Edge { child, path } => {
                let mut length = [0; 32];
                // Safe as len() is guaranteed to be <= 251
                length[31] = path.len() as u8;
                let path = Felt::from_bytes_be_slice(path);

                let length = Felt::from_bytes_be(&length);
                H::hash(child, &path) + length
            }
        }
    }
}

/// Holds the data and proofs for a specific contract.
#[derive(Debug, Serialize, Deserialize)]
pub struct ContractData {
    /// Required to verify the contract state hash to contract root calculation.
    class_hash: Felt,
    /// Required to verify the contract state hash to contract root calculation.
    nonce: Felt,

    /// Root of the Contract state tree
    root: Felt,

    /// This is currently just a constant = 0, however it might change in the
    /// future.
    contract_state_hash_version: Felt,

    /// The proofs associated with the queried storage values
    storage_proofs: Vec<Vec<TrieNode>>,
}
