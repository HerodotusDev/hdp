//! This provider is currently not used.

use std::collections::HashMap;

pub type RlpEncodedValue = String;
pub type MPTProof = Vec<String>;

/// `StoredMMR` is a tuple of root, size, and peaks.
pub type StoredMMR = (String, u64, Vec<String>);

/// `StoredMMR` is a map of mmr_id to root, size, and peaks.
type StoredMMRs = HashMap<u64, StoredMMR>;

/// `StoredHeader` is a tuple of RLP encoded header and MMR proof and element_index and mmr_id.
pub type StoredHeader = (RlpEncodedValue, MPTProof, u64, u64);

/// `StoredHeader` is a map of block number to a tuple of RLP encoded header and MMR proof and element_index and mmr_id.
pub type StoredHeaders = HashMap<u64, StoredHeader>;

/// `StoredAccount` is a map of account address to a tuple of RLP encoded account, MPT proof and stored storage.
type StoredAccounts = HashMap<String, (RlpEncodedValue, MPTProof, StoredStorages)>;

/// `StoredStorage` is a map of storage slot to a tuple of value and MPT proof.
type StoredStorages = HashMap<String, (String, MPTProof)>;

/// [`InMemoryProvider`] is a memoizer that stores the data in memory.
pub struct InMemoryProvider {
    pub cached_headers: StoredHeaders,
    pub cached_accounts: HashMap<u64, StoredAccounts>,
    pub cached_mmrs: StoredMMRs,
}

impl Default for InMemoryProvider {
    fn default() -> InMemoryProvider {
        InMemoryProvider::new()
    }
}

impl InMemoryProvider {
    pub fn new() -> InMemoryProvider {
        InMemoryProvider {
            cached_headers: HashMap::new(),
            cached_accounts: HashMap::new(),
            cached_mmrs: HashMap::new(),
        }
    }

    /// Create a memoizer with pre-filled data
    /// * Note: This is used for testing
    #[allow(dead_code)]
    fn pre_filled_memoizer(
        cached_headers: StoredHeaders,
        cached_accounts: HashMap<u64, StoredAccounts>,
        cached_mmrs: StoredMMRs,
    ) -> InMemoryProvider {
        InMemoryProvider {
            cached_headers,
            cached_accounts,
            cached_mmrs,
        }
    }

    #[allow(dead_code)]
    pub fn get_mmr(&self, mmr_id: u64) -> Option<(String, u64, Vec<String>)> {
        self.cached_mmrs.get(&mmr_id).cloned()
    }

    #[allow(dead_code)]
    pub fn get_full_headers(
        &self,
        block_numbers: Vec<u64>,
    ) -> Vec<Option<(RlpEncodedValue, MPTProof, u64, u64)>> {
        block_numbers
            .iter()
            .map(|block_number| self.get_full_header_with_proof(*block_number))
            .collect()
    }

    pub fn get_full_header_with_proof(
        &self,
        block_number: u64,
    ) -> Option<(RlpEncodedValue, MPTProof, u64, u64)> {
        self.cached_headers
            .get(&block_number)
            .map(|(header, proof, element_index, tree_id)| {
                (header.clone(), proof.clone(), *element_index, *tree_id)
            })
    }

    /// Get RLP encoded headers from the memoizer
    /// Returns a vector of `Option<RlpEncodedValue>` for each block number
    #[allow(dead_code)]
    pub fn get_rlp_headers(&self, block_numbers: Vec<u64>) -> Vec<Option<RlpEncodedValue>> {
        block_numbers
            .iter()
            .map(|block_number| self.get_rlp_header(*block_number))
            .collect()
    }

    pub fn get_rlp_header(&self, block_number: u64) -> Option<RlpEncodedValue> {
        self.cached_headers
            .get(&block_number)
            .map(|(header, _, _, _)| header.clone())
    }

    /// Get RLP encoded accounts and account proofs from the memoizer for multiple block numbers
    #[allow(dead_code)]
    pub fn get_account_from_blocks(
        &self,
        block_numbers: Vec<u64>,
        account: String,
    ) -> Vec<Option<(RlpEncodedValue, MPTProof)>> {
        block_numbers
            .iter()
            .map(|block_number| self.get_account(*block_number, &account))
            .collect()
    }

    /// Get RLP encoded account and account proof from the memoizer
    pub fn get_account(
        &self,
        block_number: u64,
        account: &str,
    ) -> Option<(RlpEncodedValue, MPTProof)> {
        self.cached_accounts
            .get(&block_number)
            .and_then(|accounts| accounts.get(account))
            .map(|(account, proof, _)| (account.clone(), proof.clone()))
    }

    /// Get only the RLP encoded account from the memoizer
    #[allow(dead_code)]
    pub fn get_rlp_account(&self, block_number: u64, account: String) -> Option<RlpEncodedValue> {
        self.cached_accounts
            .get(&block_number)
            .and_then(|accounts| accounts.get(&account))
            .map(|(account, _, _)| account.clone())
    }

    /// Get RLP encoded storages and storage proofs from the memoizer for multiple block numbers
    #[allow(dead_code)]
    pub fn get_storage_from_blocks(
        &self,
        block_numbers: Vec<u64>,
        account: String,
        slot: String,
    ) -> Vec<Option<(String, MPTProof)>> {
        block_numbers
            .iter()
            .map(|block_number| self.get_storage(*block_number, account.clone(), slot.clone()))
            .collect()
    }

    /// Get RLP encoded storage and storage proof from the memoizer
    pub fn get_storage(
        &self,
        block_number: u64,
        account: String,
        slot: String,
    ) -> Option<(String, MPTProof)> {
        self.cached_accounts
            .get(&block_number)
            .and_then(|accounts| accounts.get(&account))
            .and_then(|(_, _, storages)| storages.get(&slot))
            .map(|(value, proof)| (value.clone(), proof.clone()))
    }

    /// Get only the storage value from the memoizer
    #[allow(dead_code)]
    pub fn get_storage_value(
        &self,
        block_number: u64,
        account: String,
        slot: String,
    ) -> Option<String> {
        self.cached_accounts
            .get(&block_number)
            .and_then(|accounts| accounts.get(&account))
            .and_then(|(_, _, storages)| storages.get(&slot))
            .map(|(value, _)| value.clone())
    }

    /// unoptimized version of get_storage_value, for testing purposes
    #[allow(dead_code)]
    pub fn set_headers(&mut self, headers: Vec<(u64, RlpEncodedValue)>) {
        for (block_number, encoded_header) in headers {
            self.set_header(block_number, encoded_header);
        }
    }

    /// for testing purposes
    pub fn set_header(&mut self, block_number: u64, encoded_header: RlpEncodedValue) {
        self.cached_headers
            .insert(block_number, (encoded_header, vec![], 0, 0));
    }

    #[allow(dead_code)]
    pub fn set_full_header_with_proof(
        &mut self,
        block_number: u64,
        encoded_header: RlpEncodedValue,
        header_proof: MPTProof,
        element_index: u64,
        tree_id: u64,
    ) {
        self.cached_headers.insert(
            block_number,
            (encoded_header, header_proof, element_index, tree_id),
        );
    }

    pub fn set_mmr_data(&mut self, mmr_id: u64, root: String, size: u64, peaks: Vec<String>) {
        self.cached_mmrs.insert(mmr_id, (root, size, peaks));
    }

    pub fn set_account(
        &mut self,
        block_number: u64,
        address: String,
        encoded_account: RlpEncodedValue,
        account_proof: MPTProof,
    ) {
        self.cached_accounts
            .entry(block_number)
            .or_default()
            .insert(address, (encoded_account, account_proof, HashMap::new()));
    }

    #[allow(clippy::too_many_arguments)]
    pub fn set_storage(
        &mut self,
        block_number: u64,
        address: String,
        encoded_account: RlpEncodedValue,
        account_proof: MPTProof,
        slot: String,
        value: String,
        storage_proof: MPTProof,
    ) {
        self.cached_accounts
            .entry(block_number)
            .or_default()
            .entry(address)
            .or_insert((encoded_account, account_proof, HashMap::new()))
            .2
            .insert(slot, (value, storage_proof));
    }
}
