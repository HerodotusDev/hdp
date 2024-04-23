use anyhow::Result;
use hdp_primitives::{
    datalake::{
        output::{Header, HeaderProof, MMRMeta},
        transactions::{
            output::{Transaction, TransactionReceipt},
            TransactionsCollection, TransactionsInBlockDatalake,
        },
        DatalakeField,
    },
    utils::u64_to_fixed_bytes32,
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
            let (full_tx_and_proof_result, last_proof) = abstract_provider
                .get_tx_with_proof_from_block(datalake.target_block, datalake.increment)
                .await?;

            for (block_number, tx_index, rlp_encoded_tx, proof) in full_tx_and_proof_result {
                let value = property.decode_field_from_rlp(&rlp_encoded_tx);
                let key_fixed_bytes = u64_to_fixed_bytes32(tx_index);

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

                aggregation_set.push(value);
            }

            // Add the last proof as a last element of the transactions
            let (block_number, tx_index, proof) = last_proof;
            let key_fixed_bytes = u64_to_fixed_bytes32(tx_index);

            println!("tx_index: {}, key fixed: {}", tx_index, key_fixed_bytes);

            transactions.push(Transaction {
                key: key_fixed_bytes.to_string(),
                block_number,
                proof,
            });
        }
        TransactionsCollection::TranasactionReceipts(property) => {
            let (full_tx_receipt_and_proof_result, last_proof) = abstract_provider
                .get_tx_receipt_with_proof_from_block(datalake.target_block, datalake.increment)
                .await?;

            for (block_number, tx_index, rlp_encoded_tx_receipt, proof) in
                full_tx_receipt_and_proof_result
            {
                let value = property.decode_field_from_rlp(&rlp_encoded_tx_receipt);
                let key_fixed_bytes = u64_to_fixed_bytes32(tx_index);

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

                aggregation_set.push(value);
            }

            // Add the last proof as a last element of the transaction_receipts
            let (block_number, tx_index, proof) = last_proof;
            let key_fixed_bytes = u64_to_fixed_bytes32(tx_index);
            println!("tx_index: {}, key fixed: {}", tx_index, key_fixed_bytes);
            transaction_receipts.push(TransactionReceipt {
                key: key_fixed_bytes.to_string(),
                block_number,
                proof,
            });
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
