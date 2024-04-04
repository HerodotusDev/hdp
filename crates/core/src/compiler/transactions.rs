use anyhow::Result;
use hdp_primitives::datalake::{
    output::{Header, HeaderProof, MMRMeta},
    transactions::{
        output::{Transaction, TransactionReceipt},
        TransactionsCollection, TransactionsDatalake,
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
    pub transactions: Vec<Transaction>,
    pub transaction_receipts: Vec<TransactionReceipt>,
    /// MMR meta data related to the headers
    pub mmr_meta: MMRMeta,
}

pub async fn compile_tx_datalake(
    datalake: TransactionsDatalake,
    provider: &Arc<RwLock<AbstractProvider>>,
) -> Result<CompiledTransactionsDatalake> {
    let abstract_provider = provider.write().await;
    let mut aggregation_set: Vec<String> = Vec::new();
    let full_tx_and_proof_result = abstract_provider
        .get_tx_with_proof_from_nonce_range(
            datalake.from_base_nonce,
            datalake.to_base_nonce,
            datalake.increment,
            datalake.address.to_string(),
        )
        .await?;

    let _target_block_number_range = full_tx_and_proof_result
        .iter()
        .map(|(block_number, _, _, _)| *block_number)
        .collect::<Vec<u64>>();
    // let range_length = target_block_number_range.len();
    // TODO: Indexer should handle array of blocks, not only sequence of blocks
    // let full_header_and_proof_result = abstract_provider
    //     .get_sequencial_full_header_with_proof(
    //         target_block_number_range[0],
    //         target_block_number_range[range_length - 1],
    //     )
    //     .await?;
    let full_header_and_proof_result = abstract_provider
        .get_sequencial_full_header_with_proof(5530433, 5530434)
        .await?;
    let mmr_meta = full_header_and_proof_result.1;
    let mut headers: Vec<Header> = vec![];
    let mut transactions: Vec<Transaction> = vec![];
    let transaction_receipts: Vec<TransactionReceipt> = vec![];
    match datalake.sampled_property {
        TransactionsCollection::Transactions(property) => {
            for (block_number, tx_index, rlp_encoded_tx, proof) in full_tx_and_proof_result {
                let value = property.decode_field_from_rlp(&rlp_encoded_tx);

                transactions.push(Transaction {
                    key: tx_index.to_string(),
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
        }
        TransactionsCollection::TranasactionReceipts(_) => {
            unimplemented!();
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
