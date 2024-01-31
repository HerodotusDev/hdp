use std::str::FromStr;

#[derive(Debug)]
pub enum AccountField {
    Nonce,
    Balance,
    StorageRoot,
    CodeHash,
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
