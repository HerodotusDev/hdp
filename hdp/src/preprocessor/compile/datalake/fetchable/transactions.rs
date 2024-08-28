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
    provider::ProofProvider,
};
use alloy::primitives::U256;
use anyhow::Result;

use std::collections::HashSet;

use super::{FetchError, Fetchable, FetchedDatalake};

impl<P> Fetchable<P> for TransactionsInBlockDatalake
where
    P: ProofProvider,
{
    async fn fetch(&self, provider: P) -> Result<FetchedDatalake, FetchError> {
        let mut aggregation_set: Vec<U256> = Vec::new();

        let (mmr_metas, headers_proofs) = provider
            .get_range_of_header_proofs(self.target_block, self.target_block, self.increment)
            .await?;

        let mut headers: HashSet<ProcessedHeader> = HashSet::new();
        let mut transactions: HashSet<ProcessedTransaction> = HashSet::new();
        let mut transaction_receipts: HashSet<ProcessedReceipt> = HashSet::new();
        let fetched_block = headers_proofs.get(&self.target_block).unwrap();

        headers.insert(ProcessedHeader::new(
            fetched_block.rlp_block_header.clone(),
            fetched_block.element_index,
            fetched_block.siblings_hashes.clone(),
        ));

        match &self.sampled_property {
            TransactionsCollection::Transactions(property) => {
                for tx in provider
                    .get_tx_with_proof_from_block(
                        self.target_block,
                        self.start_index,
                        self.end_index,
                        self.increment,
                    )
                    .await?
                {
                    transactions.insert(ProcessedTransaction::new(
                        tx.tx_index,
                        tx.block_number,
                        tx.transaction_proof,
                    ));

                    // depends on datalake.included_types filter the value to be included in the aggregation set
                    if self.included_types.is_included(tx.tx_type) {
                        let value = property.decode_field_from_rlp(&tx.encoded_transaction);
                        aggregation_set.push(value);
                    }
                }
            }
            TransactionsCollection::TranasactionReceipts(property) => {
                for tx_receipt in provider
                    .get_tx_receipt_with_proof_from_block(
                        self.target_block,
                        self.start_index,
                        self.end_index,
                        self.increment,
                    )
                    .await?
                {
                    transaction_receipts.insert(ProcessedReceipt::new(
                        tx_receipt.tx_index,
                        tx_receipt.block_number,
                        tx_receipt.receipt_proof,
                    ));

                    // depends on datalake.included_types filter the value to be included in the aggregation set
                    if self.included_types.is_included(tx_receipt.tx_type) {
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
