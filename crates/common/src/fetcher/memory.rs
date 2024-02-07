use std::collections::HashMap;

pub type RlpEncodedValue = String;
pub type MPTProof = Vec<String>;

/// `MemoryFetcher` is a memoizer that stores the data in memory.
///
/// TODO: MMR proof for headers
pub struct MemoryFetcher {
    pub cached_headers: HashMap<u64, RlpEncodedValue>,
    pub cached_accounts: HashMap<u64, StoredAccounts>,
}

/// `StoredAccount` is a map of account address to a tuple of RLP encoded account, MPT proof and stored storage.
type StoredAccounts = HashMap<String, (RlpEncodedValue, MPTProof, StoredStorages)>;

/// `StoredStorage` is a map of storage slot to a tuple of value and MPT proof.
type StoredStorages = HashMap<String, (String, MPTProof)>;

impl MemoryFetcher {
    pub fn new() -> MemoryFetcher {
        MemoryFetcher {
            cached_headers: HashMap::new(),
            cached_accounts: HashMap::new(),
        }
    }

    /// Create a memoizer with pre-filled data
    /// * Note: This is used for testing
    pub fn pre_filled_memoizer(
        cached_headers: HashMap<u64, RlpEncodedValue>,
        cached_accounts: HashMap<u64, StoredAccounts>,
    ) -> MemoryFetcher {
        MemoryFetcher {
            cached_headers,
            cached_accounts,
        }
    }

    /// Get RLP encoded headers from the memoizer
    /// Returns a vector of `Option<RlpEncodedValue>` for each block number
    pub fn get_rlp_headers(&self, block_numbers: Vec<u64>) -> Vec<Option<RlpEncodedValue>> {
        block_numbers
            .iter()
            .map(|block_number| self.get_rlp_header(*block_number))
            .collect()
    }

    pub fn get_rlp_header(&self, block_number: u64) -> Option<RlpEncodedValue> {
        self.cached_headers.get(&block_number).cloned()
    }

    /// Get RLP encoded accounts and account proofs from the memoizer for multiple block numbers
    pub fn get_account_from_blocks(
        &self,
        block_numbers: Vec<u64>,
        account: String,
    ) -> Vec<Option<(RlpEncodedValue, MPTProof)>> {
        block_numbers
            .iter()
            .map(|block_number| self.get_account(*block_number, account.clone()))
            .collect()
    }

    /// Get RLP encoded account and account proof from the memoizer
    pub fn get_account(
        &self,
        block_number: u64,
        account: String,
    ) -> Option<(RlpEncodedValue, MPTProof)> {
        self.cached_accounts
            .get(&block_number)
            .and_then(|accounts| accounts.get(&account))
            .map(|(account, proof, _)| (account.clone(), proof.clone()))
    }

    /// Get only the RLP encoded account from the memoizer
    pub fn get_rlp_account(&self, block_number: u64, account: String) -> Option<RlpEncodedValue> {
        self.cached_accounts
            .get(&block_number)
            .and_then(|accounts| accounts.get(&account))
            .map(|(account, _, _)| account.clone())
    }

    /// Get RLP encoded storages and storage proofs from the memoizer for multiple block numbers
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

    /// Set RLP encoded headers in the memoizer
    pub fn set_headers(&mut self, headers: Vec<(u64, RlpEncodedValue)>) {
        for (block_number, encoded_header) in headers {
            self.set_header(block_number, encoded_header);
        }
    }

    pub fn set_header(&mut self, block_number: u64, encoded_header: RlpEncodedValue) {
        self.cached_headers.insert(block_number, encoded_header);
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

impl Default for MemoryFetcher {
    fn default() -> MemoryFetcher {
        MemoryFetcher::new()
    }
}
