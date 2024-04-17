use std::{fmt, sync::Arc};

use anyhow::{bail, Result};
use hdp_primitives::datalake::{
    block_sampled::output::{Account, Storage},
    envelope::DatalakeEnvelope,
    output::{Header, MMRMeta},
    transactions::output::{Transaction, TransactionReceipt},
};
use hdp_provider::evm::AbstractProvider;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use self::{
    block_sampled::{compile_block_sampled_datalake, CompiledBlockSampledDatalake},
    transactions::{compile_tx_datalake, CompiledTransactionsDatalake},
};

pub mod block_sampled;
pub mod test;
pub mod transactions;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CompiledDatalakeEnvelope {
    /// Block sampled datalake
    BlockSampled(CompiledBlockSampledDatalake),
    /// Transactions datalake
    Transactions(CompiledTransactionsDatalake),
}

impl CompiledDatalakeEnvelope {
    ///Get values from compiled datalake
    pub fn get_values(&self) -> Vec<String> {
        match self {
            CompiledDatalakeEnvelope::BlockSampled(compiled_block_sampled_datalake) => {
                compiled_block_sampled_datalake.values.clone()
            }
            CompiledDatalakeEnvelope::Transactions(compiled_transactions_datalake) => {
                compiled_transactions_datalake.values.clone()
            }
        }
    }

    ///Get headers from compiled datalake
    pub fn get_headers(&self) -> Vec<Header> {
        match self {
            CompiledDatalakeEnvelope::BlockSampled(compiled_block_sampled_datalake) => {
                compiled_block_sampled_datalake.headers.clone()
            }
            CompiledDatalakeEnvelope::Transactions(compiled_transactions_datalake) => {
                compiled_transactions_datalake.headers.clone()
            }
        }
    }

    ///Get account from compiled datalake
    pub fn get_accounts(&self) -> Result<Vec<Account>> {
        match self {
            CompiledDatalakeEnvelope::BlockSampled(compiled_block_sampled_datalake) => {
                Ok(compiled_block_sampled_datalake.accounts.clone())
            }
            CompiledDatalakeEnvelope::Transactions(_) => {
                bail!("transactions datalake does not have accounts")
            }
        }
    }

    /// Get storages from compiled datalake
    pub fn get_storages(&self) -> Result<Vec<Storage>> {
        match self {
            CompiledDatalakeEnvelope::BlockSampled(compiled_block_sampled_datalake) => {
                Ok(compiled_block_sampled_datalake.storages.clone())
            }
            CompiledDatalakeEnvelope::Transactions(_) => {
                bail!("transactions datalake does not have storages")
            }
        }
    }

    /// Get transactions from compiled datalake
    pub fn get_transactions(&self) -> Result<Vec<Transaction>> {
        match self {
            CompiledDatalakeEnvelope::BlockSampled(_) => {
                bail!("block sampled datalake does not have transactions")
            }
            CompiledDatalakeEnvelope::Transactions(compiled_transactions_datalake) => {
                Ok(compiled_transactions_datalake.transactions.clone())
            }
        }
    }

    /// Get transaction receipts from compiled datalake
    pub fn get_transaction_receipts(&self) -> Result<Vec<TransactionReceipt>> {
        match self {
            CompiledDatalakeEnvelope::BlockSampled(_) => {
                bail!("block sampled datalake does not have transaction receipts")
            }
            CompiledDatalakeEnvelope::Transactions(compiled_transactions_datalake) => {
                Ok(compiled_transactions_datalake.transaction_receipts.clone())
            }
        }
    }

    /// Get mmr_meta from compiled datalake
    pub fn get_mmr_meta(&self) -> Result<MMRMeta> {
        match self {
            CompiledDatalakeEnvelope::BlockSampled(compiled_block_sampled_datalake) => {
                Ok(compiled_block_sampled_datalake.mmr_meta.clone())
            }
            CompiledDatalakeEnvelope::Transactions(compiled_transactions_datalake) => {
                Ok(compiled_transactions_datalake.mmr_meta.clone())
            }
        }
    }
}

pub struct DatalakeCompiler {
    /// Datalake commitment. It is used to identify the datalake
    pub commitment: String,
    /// Datalake
    pub datalake: DatalakeEnvelope,
}

impl fmt::Debug for DatalakeCompiler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DatalakeCompiler")
            .field("commitment", &self.commitment)
            .field("datalakes_pipeline", &self.datalake)
            .finish()
    }
}

impl DatalakeCompiler {
    /// initialize DatalakeCompiler with commitment and datalake
    pub fn new(datalake: DatalakeEnvelope) -> Self {
        Self {
            commitment: datalake.get_commitment(),
            datalake,
        }
    }

    /// Compile the datalake meaning, fetching relevant headers, accounts, storages, and mmr_meta data.
    ///
    /// Plus, it will combine target datalake's datapoints in compiled_results.
    pub async fn compile(
        &self,
        provider: &Arc<RwLock<AbstractProvider>>,
    ) -> Result<CompiledDatalakeEnvelope> {
        let result_datapoints = match &self.datalake {
            DatalakeEnvelope::BlockSampled(datalake) => CompiledDatalakeEnvelope::BlockSampled(
                compile_block_sampled_datalake(datalake.clone(), provider).await?,
            ),
            DatalakeEnvelope::Transactions(datalake) => CompiledDatalakeEnvelope::Transactions(
                compile_tx_datalake(datalake.clone(), provider).await?,
            ),
        };

        Ok(result_datapoints)
    }
}
