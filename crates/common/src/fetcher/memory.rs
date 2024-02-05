use std::collections::HashMap;

pub type RlpEncodedValue = String;

pub struct MemoryFetcher {
    pub headers: HashMap<usize, String>,
    pub accounts: HashMap<usize, HashMap<String, String>>,
    pub storages: HashMap<usize, HashMap<String, HashMap<String, String>>>,
}

impl MemoryFetcher {
    pub fn new() -> MemoryFetcher {
        MemoryFetcher {
            headers: HashMap::new(),
            accounts: HashMap::new(),
            storages: HashMap::new(),
        }
    }

    /// Create a memoizer with pre-filled data
    /// * Note: This is used for testing
    pub fn pre_filled_memoizer(
        headers: HashMap<usize, RlpEncodedValue>,
        accounts: HashMap<usize, HashMap<String, RlpEncodedValue>>,
        storages: HashMap<usize, HashMap<String, HashMap<String, String>>>,
    ) -> MemoryFetcher {
        MemoryFetcher {
            headers,
            accounts,
            storages,
        }
    }

    pub fn get_rlp_header(&self, block_number: usize) -> Option<RlpEncodedValue> {
        self.headers.get(&block_number).cloned()
    }

    pub fn get_rlp_account(&self, block_number: usize, account: String) -> Option<RlpEncodedValue> {
        self.accounts
            .get(&block_number)
            .and_then(|accounts| accounts.get(&account).cloned())
    }

    pub fn get_storage_value(
        &self,
        block_number: usize,
        account: String,
        slot: String,
    ) -> Option<String> {
        self.storages
            .get(&block_number)
            .and_then(|storages| storages.get(&account))
            .and_then(|slots| slots.get(&slot).cloned())
    }

    pub fn set_header(&mut self, block_number: usize, header: RlpEncodedValue) {
        self.headers.insert(block_number, header);
    }

    pub fn set_account(&mut self, block_number: usize, account: String, value: RlpEncodedValue) {
        let accounts = self.accounts.entry(block_number).or_default();
        accounts.insert(account, value);
    }

    pub fn set_storage(
        &mut self,
        block_number: usize,
        account: String,
        slot: String,
        value: String,
    ) {
        let storages = self.storages.entry(block_number).or_default();
        let slots = storages.entry(account).or_default();
        slots.insert(slot, value);
    }
}

impl Default for MemoryFetcher {
    fn default() -> MemoryFetcher {
        MemoryFetcher::new()
    }
}
