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
    utils::tx_index_to_tx_key,
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
            for tx in abstract_provider
                .get_tx_with_proof_from_block(
                    datalake.target_block,
                    datalake.start_index,
                    datalake.end_index,
                    datalake.increment,
                )
                .await?
            {
                let key_fixed_bytes = tx_index_to_tx_key(tx.tx_index);

                transactions.push(Transaction {
                    key: key_fixed_bytes.to_string(),
                    block_number: tx.block_number,
                    proof: tx.transaction_proof,
                });

                headers.push(Header {
                    rlp: full_header_and_proof_result
                        .0
                        .get(&tx.block_number)
                        .unwrap()
                        .0
                        .clone(),
                    proof: HeaderProof {
                        leaf_idx: full_header_and_proof_result
                            .0
                            .get(&tx.block_number)
                            .unwrap()
                            .2,
                        mmr_path: full_header_and_proof_result
                            .0
                            .get(&tx.block_number)
                            .unwrap()
                            .1
                            .clone(),
                    },
                });

                // depends on datalake.included_types filter the value to be included in the aggregation set
                if datalake.included_types.is_included(tx.tx_type) {
                    let value = property.decode_field_from_rlp(&tx.encoded_transaction);
                    aggregation_set.push(value);
                }
            }
        }
        TransactionsCollection::TranasactionReceipts(property) => {
            for tx_receipt in abstract_provider
                .get_tx_receipt_with_proof_from_block(
                    datalake.target_block,
                    datalake.start_index,
                    datalake.end_index,
                    datalake.increment,
                )
                .await?
            {
                let key_fixed_bytes = tx_index_to_tx_key(tx_receipt.tx_index);

                transaction_receipts.push(TransactionReceipt {
                    key: key_fixed_bytes.to_string(),
                    block_number: tx_receipt.block_number,
                    proof: tx_receipt.receipt_proof,
                });

                headers.push(Header {
                    rlp: full_header_and_proof_result
                        .0
                        .get(&tx_receipt.block_number)
                        .unwrap()
                        .0
                        .clone(),
                    proof: HeaderProof {
                        leaf_idx: full_header_and_proof_result
                            .0
                            .get(&tx_receipt.block_number)
                            .unwrap()
                            .2,
                        mmr_path: full_header_and_proof_result
                            .0
                            .get(&tx_receipt.block_number)
                            .unwrap()
                            .1
                            .clone(),
                    },
                });

                // depends on datalake.included_types filter the value to be included in the aggregation set
                if datalake.included_types.is_included(tx_receipt.tx_type) {
                    let value = property.decode_field_from_rlp(&tx_receipt.encoded_receipt);
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