use crate::{
    primitives::{
        processed_types::{
            header::ProcessedHeader, receipt::ProcessedReceipt, transaction::ProcessedTransaction,
        },
        task::datalake::{
            transactions::{TransactionsCollection, TransactionsInBlockDatalake},
            DatalakeField,
        },
    },
    provider::{error::ProviderError, evm::provider::EvmProvider, types::FetchedDatalake},
};
use alloy::primitives::U256;
use anyhow::Result;

use std::collections::HashSet;

impl EvmProvider {
    pub async fn fetch_transactions(
        &self,
        datalake: &TransactionsInBlockDatalake,
    ) -> Result<FetchedDatalake, ProviderError> {
        let mut aggregation_set: Vec<U256> = Vec::new();

        let (mmr_metas, headers_proofs) = self
            .get_range_of_header_proofs(
                datalake.target_block,
                datalake.target_block,
                datalake.increment,
            )
            .await?;

        let mut headers: HashSet<ProcessedHeader> = HashSet::new();
        let mut transactions: HashSet<ProcessedTransaction> = HashSet::new();
        let mut transaction_receipts: HashSet<ProcessedReceipt> = HashSet::new();
        let fetched_block = headers_proofs.get(&datalake.target_block).unwrap();

        headers.insert(ProcessedHeader::new(
            fetched_block.rlp_block_header.clone(),
            fetched_block.element_index,
            fetched_block.siblings_hashes.clone(),
        ));

        match &datalake.sampled_property {
            TransactionsCollection::Transactions(property) => {
                for tx in self
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
                for tx_receipt in self
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

        Ok(FetchedDatalake {
            values: aggregation_set,
            headers,
            accounts: HashSet::new(),
            storages: HashSet::new(),
            transactions,
            transaction_receipts,
            mmr_metas,
        })
    }
}
