use alloy_primitives::{FixedBytes, U256};
use anyhow::Result;
use hdp_primitives::datalake::{
    output::{Header, HeaderProof, MMRMeta},
    transactions::{
        output::{Transaction, TransactionReceipt},
        TransactionsCollection, TransactionsInBlockDatalake,
    },
    DatalakeField,
};
use hdp_provider::evm::AbstractProvider;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledTransactionsDatalake {
    /// Targeted datalake's compiled results
    pub values: Vec<String>,
    /// Headers related to the datalake
    pub headers: Vec<Header>,
    /// Transactions related to the datalake
    pub transactions: Vec<Transaction>,
    /// Transaction receipts related to the datalake
    pub transaction_receipts: Vec<TransactionReceipt>,
    /// MMR meta data related to the headers
    pub mmr_meta: MMRMeta,
}

pub async fn compile_tx_datalake(
    datalake: TransactionsInBlockDatalake,
    provider: &Arc<RwLock<AbstractProvider>>,
) -> Result<CompiledTransactionsDatalake> {
    let abstract_provider = provider.write().await;
    let mut aggregation_set: Vec<String> = Vec::new();

    let full_header_and_proof_result = abstract_provider
        .get_sequencial_full_header_with_proof(datalake.target_block, datalake.target_block)
        .await?;
    let mmr_meta = full_header_and_proof_result.1;
    let mut headers: Vec<Header> = vec![];
    let mut transactions: Vec<Transaction> = vec![];
    let mut transaction_receipts: Vec<TransactionReceipt> = vec![];

    match datalake.sampled_property {
        TransactionsCollection::Transactions(property) => {
            for (block_number, tx_index, rlp_encoded_tx, proof, tx_type) in abstract_provider
                .get_tx_with_proof_from_block(
                    datalake.target_block,
                    datalake.start_index,
                    datalake.end_index,
                    datalake.increment,
                )
                .await?
            {
                let value = property.decode_field_from_rlp(&rlp_encoded_tx);

                let key_fixed_bytes = tx_index_to_tx_key(tx_index);

                transactions.push(Transaction {
                    key: key_fixed_bytes.to_string(),
                    block_number,
                    proof,
                });

                headers.push(Header {
                    rlp: full_header_and_proof_result
                        .0
                        .get(&block_number)
                        .unwrap()
                        .0
                        .clone(),
                    proof: HeaderProof {
                        leaf_idx: full_header_and_proof_result.0.get(&block_number).unwrap().2,
                        mmr_path: full_header_and_proof_result
                            .0
                            .get(&block_number)
                            .unwrap()
                            .1
                            .clone(),
                    },
                });

                // depends on datalake.included_types filter the value to be included in the aggregation set
                if datalake.included_types.is_included(tx_type) {
                    aggregation_set.push(value);
                }
            }
        }
        TransactionsCollection::TranasactionReceipts(property) => {
            for (block_number, tx_index, rlp_encoded_tx_receipt, proof, tx_type) in
                abstract_provider
                    .get_tx_receipt_with_proof_from_block(
                        datalake.target_block,
                        datalake.start_index,
                        datalake.end_index,
                        datalake.increment,
                    )
                    .await?
            {
                let value = property.decode_field_from_rlp(&rlp_encoded_tx_receipt);
                let key_fixed_bytes = tx_index_to_tx_key(tx_index);

                transaction_receipts.push(TransactionReceipt {
                    key: key_fixed_bytes.to_string(),
                    block_number,
                    proof,
                });

                headers.push(Header {
                    rlp: full_header_and_proof_result
                        .0
                        .get(&block_number)
                        .unwrap()
                        .0
                        .clone(),
                    proof: HeaderProof {
                        leaf_idx: full_header_and_proof_result.0.get(&block_number).unwrap().2,
                        mmr_path: full_header_and_proof_result
                            .0
                            .get(&block_number)
                            .unwrap()
                            .1
                            .clone(),
                    },
                });

                // depends on datalake.included_types filter the value to be included in the aggregation set
                if datalake.included_types.is_included(tx_type) {
                    aggregation_set.push(value);
                }
            }
        }
    }

    Ok(CompiledTransactionsDatalake {
        values: aggregation_set,
        headers,
        transactions,
        transaction_receipts,
        mmr_meta,
    })
}

/// Convert a transaction index to a transaction key
fn tx_index_to_tx_key(tx_index: u64) -> String {
    let binding = alloy_rlp::encode(U256::from(tx_index));
    let tx_index_bytes = binding.as_slice();
    let mut buffer = [0u8; 32];
    let start = 32 - tx_index_bytes.len();
    buffer[start..].copy_from_slice(tx_index_bytes);
    FixedBytes::from(buffer).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hdp_primitives::datalake::output::{split_big_endian_hex_into_parts, Uint256};

    #[test]
    fn test_tx_index_to_tx_key() {
        // no rlp prefix
        let tx_index = 127u64;
        let tx_key = tx_index_to_tx_key(tx_index);
        let expected_tx_key =
            "0x000000000000000000000000000000000000000000000000000000000000007f".to_string();

        assert_eq!(tx_key, expected_tx_key);

        let split = split_big_endian_hex_into_parts(&expected_tx_key);
        assert_eq!(
            split,
            Uint256 {
                low: "0x0000000000000000000000000000007f".to_string(),
                high: "0x00000000000000000000000000000000".to_string(),
            }
        );

        // rlpx prefix
        let tx_index = 303u64;
        let tx_key = tx_index_to_tx_key(tx_index);
        let expected_tx_key =
            "0x000000000000000000000000000000000000000000000000000000000082012f".to_string();

        assert_eq!(tx_key, expected_tx_key);

        let split = split_big_endian_hex_into_parts(&expected_tx_key);
        assert_eq!(
            split,
            Uint256 {
                low: "0x0000000000000000000000000082012f".to_string(),
                high: "0x00000000000000000000000000000000".to_string(),
            }
        )
    }
}
