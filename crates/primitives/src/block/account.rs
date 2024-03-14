use anyhow::{bail, Result};
use std::str::FromStr;

use alloy_primitives::{hex, FixedBytes, U256};
use alloy_rlp::{Decodable, Encodable as _, RlpDecodable, RlpEncodable};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum AccountField {
    Nonce,
    Balance,
    StorageRoot,
    CodeHash,
}

#[derive(Debug, RlpDecodable, RlpEncodable, PartialEq)]
pub struct Account {
    pub nonce: u64,
    pub balance: U256,
    pub storage_root: FixedBytes<32>,
    pub code_hash: FixedBytes<32>,
}

impl Account {
    pub fn new(
        nonce: u64,
        balance: U256,
        storage_root: FixedBytes<32>,
        code_hash: FixedBytes<32>,
    ) -> Self {
        Account {
            nonce,
            balance,
            storage_root,
            code_hash,
        }
    }

    pub fn rlp_encode(&self) -> String {
        let mut buffer = Vec::<u8>::new();
        self.encode(&mut buffer);
        hex::encode(buffer)
    }

    pub fn rlp_decode(rlp: &str) -> Self {
        let decoded = <Account>::decode(&mut hex::decode(rlp).unwrap().as_slice()).unwrap();
        decoded
    }
}

impl FromStr for AccountField {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "NONCE" => Ok(AccountField::Nonce),
            "BALANCE" => Ok(AccountField::Balance),
            "STORAGE_ROOT" => Ok(AccountField::StorageRoot),
            "CODE_HASH" => Ok(AccountField::CodeHash),
            _ => bail!("Unknown account field"),
        }
    }
}

impl AccountField {
    pub fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(AccountField::Nonce),
            1 => Some(AccountField::Balance),
            2 => Some(AccountField::StorageRoot),
            3 => Some(AccountField::CodeHash),
            _ => None,
        }
    }

    pub fn to_index(&self) -> u8 {
        match self {
            AccountField::Nonce => 0,
            AccountField::Balance => 1,
            AccountField::StorageRoot => 2,
            AccountField::CodeHash => 3,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AccountField::Nonce => "NONCE",
            AccountField::Balance => "BALANCE",
            AccountField::StorageRoot => "STORAGE_ROOT",
            AccountField::CodeHash => "CODE_HASH",
        }
    }
}

pub fn decode_account_field(account_rlp: &str, field: AccountField) -> String {
    let decoded = <Account>::decode(&mut hex::decode(account_rlp).unwrap().as_slice()).unwrap();
    match field {
        AccountField::Nonce => decoded.nonce.to_string(),
        AccountField::Balance => decoded.balance.to_string(),
        AccountField::StorageRoot => decoded.storage_root.to_string(),
        AccountField::CodeHash => decoded.code_hash.to_string(),
    }
}

/// Account data from RPC `eth_getProof` response
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccountFromRpc {
    pub account_proof: Vec<String>,
    pub address: String,
    pub balance: String,
    pub code_hash: String,
    pub nonce: String,
    pub storage_hash: String,
    pub storage_proof: Vec<StorageFromRpc>,
}

/// Storage data from RPC `eth_getProof` response
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StorageFromRpc {
    pub key: String,
    pub proof: Vec<String>,
    pub value: String,
}

impl From<&AccountFromRpc> for Account {
    fn from(account_from_rpc: &AccountFromRpc) -> Self {
        Account {
            nonce: u64::from_str_radix(&account_from_rpc.nonce[2..], 16).unwrap(),
            balance: U256::from_str_radix(&account_from_rpc.balance[2..], 16).unwrap(),
            storage_root: FixedBytes::from_str(&account_from_rpc.storage_hash[2..]).unwrap(),
            code_hash: FixedBytes::from_str(&account_from_rpc.code_hash[2..]).unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{FixedBytes, U256};
    use std::str::FromStr;

    #[test]
    fn test_get_account_rlp() {
        // let account_addr = "0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6";
        let account = Account::new(
            1,
            U256::from(0),
            FixedBytes::from_str(
                "0x1c35dfde2b62d99d3a74fda76446b60962c4656814bdd7815eb6e5b8be1e7185",
            )
            .unwrap(),
            FixedBytes::from_str(
                "0xcd4f25236fff0ccac15e82bf4581beb08e95e1b5ba89de6031c75893cd91245c",
            )
            .unwrap(),
        );
        let account_rlp = account.rlp_encode();
        assert_eq!(account_rlp, "f8440180a01c35dfde2b62d99d3a74fda76446b60962c4656814bdd7815eb6e5b8be1e7185a0cd4f25236fff0ccac15e82bf4581beb08e95e1b5ba89de6031c75893cd91245c");
        let account = Account::new(
            2,
            U256::from(0),
            FixedBytes::from_str(
                "0x1c35dfde2b62d99d3a74fda76446b60962c4656814bdd7815eb6e5b8be1e7185",
            )
            .unwrap(),
            FixedBytes::from_str(
                "0xcd4f25236fff0ccac15e82bf4581beb08e95e1b5ba89de6031c75893cd91245c",
            )
            .unwrap(),
        );
        let account_rlp = account.rlp_encode();
        assert_eq!(account_rlp, "f8440280a01c35dfde2b62d99d3a74fda76446b60962c4656814bdd7815eb6e5b8be1e7185a0cd4f25236fff0ccac15e82bf4581beb08e95e1b5ba89de6031c75893cd91245c");
        let account = Account::new(
            2,
            U256::from(0x1),
            FixedBytes::from_str(
                "0x1c35dfde2b62d99d3a74fda76446b60962c4656814bdd7815eb6e5b8be1e7185",
            )
            .unwrap(),
            FixedBytes::from_str(
                "0xcd4f25236fff0ccac15e82bf4581beb08e95e1b5ba89de6031c75893cd91245c",
            )
            .unwrap(),
        );
        let account_rlp = account.rlp_encode();
        assert_eq!(account_rlp, "f8440201a01c35dfde2b62d99d3a74fda76446b60962c4656814bdd7815eb6e5b8be1e7185a0cd4f25236fff0ccac15e82bf4581beb08e95e1b5ba89de6031c75893cd91245c");
    }

    #[test]
    fn test_decode_account_rlp() {
        let account_rlp = "f8440180a01c35dfde2b62d99d3a74fda76446b60962c4656814bdd7815eb6e5b8be1e7185a0cd4f25236fff0ccac15e82bf4581beb08e95e1b5ba89de6031c75893cd91245c";
        let account = Account::rlp_decode(account_rlp);
        assert_eq!(
            account,
            Account::new(
                1,
                U256::from(0),
                FixedBytes::from_str(
                    "0x1c35dfde2b62d99d3a74fda76446b60962c4656814bdd7815eb6e5b8be1e7185"
                )
                .unwrap(),
                FixedBytes::from_str(
                    "0xcd4f25236fff0ccac15e82bf4581beb08e95e1b5ba89de6031c75893cd91245c"
                )
                .unwrap()
            )
        );
    }
}
