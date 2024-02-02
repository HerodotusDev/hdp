use std::str::FromStr;

use alloy_primitives::{hex, FixedBytes, U256};
use alloy_rlp::{Decodable, Encodable as _, RlpDecodable, RlpEncodable};

use crate::datalake::base::DataPoint;

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
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NONCE" => Ok(AccountField::Nonce),
            "BALANCE" => Ok(AccountField::Balance),
            "STORAGE_ROOT" => Ok(AccountField::StorageRoot),
            "CODE_HASH" => Ok(AccountField::CodeHash),
            _ => Err(()),
        }
    }
}

impl AccountField {
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(AccountField::Nonce),
            1 => Some(AccountField::Balance),
            2 => Some(AccountField::StorageRoot),
            3 => Some(AccountField::CodeHash),
            _ => None,
        }
    }

    pub fn to_index(&self) -> Option<usize> {
        match self {
            AccountField::Nonce => Some(0),
            AccountField::Balance => Some(1),
            AccountField::StorageRoot => Some(2),
            AccountField::CodeHash => Some(3),
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

pub fn decode_account_field(account_rlp: &str, field: AccountField) -> DataPoint {
    let decoded = <Account>::decode(&mut hex::decode(account_rlp).unwrap().as_slice()).unwrap();
    match field {
        AccountField::Nonce => DataPoint::Int(u64::from_str(&decoded.nonce.to_string()).unwrap()),
        AccountField::Balance => {
            DataPoint::Int(u64::from_str(&decoded.balance.to_string()).unwrap())
        }
        AccountField::StorageRoot => DataPoint::Str(decoded.storage_root.to_string()),
        AccountField::CodeHash => DataPoint::Str(decoded.code_hash.to_string()),
    }
}
