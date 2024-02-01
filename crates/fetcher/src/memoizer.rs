use std::collections::HashMap;

pub type RlpEncodedValue = String;

pub struct Memoizer {
    pub headers: HashMap<usize, String>,
    pub accounts: HashMap<usize, HashMap<String, String>>,
    pub storages: HashMap<usize, HashMap<String, HashMap<String, String>>>,
}

impl Memoizer {
    pub fn new() -> Memoizer {
        Memoizer {
            headers: HashMap::new(),
            accounts: HashMap::new(),
            storages: HashMap::new(),
        }
    }

    pub fn pre_filled_memoizer(
        headers: HashMap<usize, RlpEncodedValue>,
        accounts: HashMap<usize, HashMap<String, RlpEncodedValue>>,
        storages: HashMap<usize, HashMap<String, HashMap<String, RlpEncodedValue>>>,
    ) -> Memoizer {
        Memoizer {
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

    pub fn get_rlp_storage(
        &self,
        block_number: usize,
        account: String,
        slot: String,
    ) -> Option<RlpEncodedValue> {
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
        value: RlpEncodedValue,
    ) {
        let storages = self.storages.entry(block_number).or_default();
        let slots = storages.entry(account).or_default();
        slots.insert(slot, value);
    }
}

impl Default for Memoizer {
    fn default() -> Memoizer {
        Memoizer::new()
    }
}
