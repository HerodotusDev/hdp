use alloy::primitives::U256;
use anyhow::Result;
use hdp_primitives::{
    datalake::{
        transactions::{TransactionsCollection, TransactionsInBlockDatalake},
        DatalakeField,
    },
    processed_types::{
        header::ProcessedHeader, mmr::MMRMeta, receipt::ProcessedReceipt,
        transaction::ProcessedTransaction,
    },
};

use hdp_provider::evm::provider::EvmProvider;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, sync::Arc};
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompiledTransactionsDatalake {
    /// Targeted datalake's compiled results
    pub values: Vec<U256>,
    /// Headers related to the datalake
    pub headers: HashSet<ProcessedHeader>,
    /// Transactions related to the datalake
    pub transactions: HashSet<ProcessedTransaction>,
    /// Transaction receipts related to the datalake
    pub transaction_receipts: HashSet<ProcessedReceipt>,
    /// MMR meta data related to the headers
    pub mmr_meta: MMRMeta,
}

pub async fn compile_tx_datalake(
    datalake: TransactionsInBlockDatalake,
    provider: &Arc<RwLock<EvmProvider>>,
) -> Result<CompiledTransactionsDatalake> {
    let abstract_provider = provider.write().await;
    let mut aggregation_set: Vec<U256> = Vec::new();

    let (mmr_meta, headers_proofs) = abstract_provider
        .get_range_of_header_proofs(
            datalake.target_block,
            datalake.target_block,
            datalake.increment,
        )
        .await?;
    let mmr_meta = MMRMeta::from(mmr_meta);
    let mut headers: HashSet<ProcessedHeader> = HashSet::new();
    let mut transactions: HashSet<ProcessedTransaction> = HashSet::new();
    let mut transaction_receipts: HashSet<ProcessedReceipt> = HashSet::new();
    let fetched_block = headers_proofs.get(&datalake.target_block).unwrap();

    headers.insert(ProcessedHeader::new(
        fetched_block.rlp_block_header.clone(),
        fetched_block.element_index,
        fetched_block.siblings_hashes.clone(),
    ));

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
                transactions.insert(ProcessedTransaction::new(
                    tx.tx_index,
                    tx.block_number,
                    tx.transaction_proof,
                ));

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
                transaction_receipts.insert(ProcessedReceipt::new(
                    tx_receipt.tx_index,
                    tx_receipt.block_number,
                    tx_receipt.receipt_proof,
                ));

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
