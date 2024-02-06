use crate::block::{account::Account, header::BlockHeader};

use self::{
    memory::{MemoryFetcher, RlpEncodedValue},
    rpc::RpcFetcher,
};

pub mod memory;
pub mod prefilled_data;
pub mod rpc;

pub struct AbstractFetcher {
    memory: MemoryFetcher,
    rpc: RpcFetcher,
}

impl AbstractFetcher {
    pub fn new(rpc_url: String) -> Self {
        Self {
            memory: MemoryFetcher::new(),
            rpc: RpcFetcher::new(rpc_url),
        }
    }

    pub async fn get_rlp_header(&mut self, block_number: usize) -> RlpEncodedValue {
        match self.memory.get_rlp_header(block_number) {
            Some(header) => header,
            None => {
                let header_rpc = self
                    .rpc
                    .get_block_by_number(block_number as u64)
                    .await
                    .unwrap();
                let block_header = BlockHeader::from(&header_rpc);
                let rlp_encoded = block_header.rlp_encode();
                self.memory.set_header(block_number, rlp_encoded.clone());
                rlp_encoded
            }
        }
    }

    pub async fn get_rlp_account(
        &mut self,
        block_number: usize,
        account: String,
    ) -> RlpEncodedValue {
        match self.memory.get_rlp_account(block_number, account.clone()) {
            Some(account) => account,
            None => {
                let account_rpc = self
                    .rpc
                    .get_proof(block_number as u64, account.clone(), None)
                    .await
                    .unwrap();
                let retrieved_account = Account::from(&account_rpc);
                let rlp_encoded = retrieved_account.rlp_encode();
                self.memory
                    .set_account(block_number, account_rpc.address, rlp_encoded.clone());
                rlp_encoded
            }
        }
    }

    pub async fn get_storage_value(
        &mut self,
        block_number: usize,
        account: String,
        slot: String,
    ) -> String {
        match self
            .memory
            .get_storage_value(block_number, account.clone(), slot.clone())
        {
            Some(storage) => storage,
            None => {
                let storage_rpc = self
                    .rpc
                    .get_proof(
                        block_number as u64,
                        account.clone(),
                        Some(vec![slot.clone()]),
                    )
                    .await
                    .unwrap();
                let storage = &storage_rpc.storage_proof[0];
                let storage_slot = storage.key.clone();
                let storage_value = storage.value.clone();
                self.memory
                    .set_storage(block_number, account, storage_slot, storage_value.clone());
                storage_value
            }
        }
    }
}
