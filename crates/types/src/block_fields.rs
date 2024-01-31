use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum Collection {
    Header,
    Account,
    Storage,
}

#[derive(Debug)]
pub enum HeaderField {
    ParentHash,
    OmmerHash,
    Beneficiary,
    StateRoot,
    TransactionsRoot,
    ReceiptsRoot,
    LogsBloom,
    Difficulty,
    Number,
    GasLimit,
    GasUsed,
    Timestamp,
    ExtraData,
    MixHash,
    Nonce,
    BaseFeePerGas,
}

impl HeaderField {
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(HeaderField::ParentHash),
            1 => Some(HeaderField::OmmerHash),
            2 => Some(HeaderField::Beneficiary),
            3 => Some(HeaderField::StateRoot),
            4 => Some(HeaderField::TransactionsRoot),
            5 => Some(HeaderField::ReceiptsRoot),
            6 => Some(HeaderField::LogsBloom),
            7 => Some(HeaderField::Difficulty),
            8 => Some(HeaderField::Number),
            9 => Some(HeaderField::GasLimit),
            10 => Some(HeaderField::GasUsed),
            11 => Some(HeaderField::Timestamp),
            12 => Some(HeaderField::ExtraData),
            13 => Some(HeaderField::MixHash),
            14 => Some(HeaderField::Nonce),
            15 => Some(HeaderField::BaseFeePerGas),
            _ => None,
        }
    }

    pub fn to_index(&self) -> Option<usize> {
        match self {
            HeaderField::ParentHash => Some(0),
            HeaderField::OmmerHash => Some(1),
            HeaderField::Beneficiary => Some(2),
            HeaderField::StateRoot => Some(3),
            HeaderField::TransactionsRoot => Some(4),
            HeaderField::ReceiptsRoot => Some(5),
            HeaderField::LogsBloom => Some(6),
            HeaderField::Difficulty => Some(7),
            HeaderField::Number => Some(8),
            HeaderField::GasLimit => Some(9),
            HeaderField::GasUsed => Some(10),
            HeaderField::Timestamp => Some(11),
            HeaderField::ExtraData => Some(12),
            HeaderField::MixHash => Some(13),
            HeaderField::Nonce => Some(14),
            HeaderField::BaseFeePerGas => Some(15),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            HeaderField::ParentHash => "PARENT_HASH",
            HeaderField::OmmerHash => "OMMERS_HASH",
            HeaderField::Beneficiary => "BENEFICIARY",
            HeaderField::StateRoot => "STATE_ROOT",
            HeaderField::TransactionsRoot => "TRANSACTIONS_ROOT",
            HeaderField::ReceiptsRoot => "RECEIPTS_ROOT",
            HeaderField::LogsBloom => "LOGS_BLOOM",
            HeaderField::Difficulty => "DIFFICULTY",
            HeaderField::Number => "NUMBER",
            HeaderField::GasLimit => "GAS_LIMIT",
            HeaderField::GasUsed => "GAS_USED",
            HeaderField::Timestamp => "TIMESTAMP",
            HeaderField::ExtraData => "EXTRA_DATA",
            HeaderField::MixHash => "MIX_HASH",
            HeaderField::Nonce => "NONCE",
            HeaderField::BaseFeePerGas => "BASE_FEE_PER_GAS",
        }
    }
}

impl FromStr for HeaderField {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PARENT_HASH" => Ok(HeaderField::ParentHash),
            "OMMERS_HASH" => Ok(HeaderField::OmmerHash),
            "BENEFICIARY" => Ok(HeaderField::Beneficiary),
            "STATE_ROOT" => Ok(HeaderField::StateRoot),
            "TRANSACTIONS_ROOT" => Ok(HeaderField::TransactionsRoot),
            "RECEIPTS_ROOT" => Ok(HeaderField::ReceiptsRoot),
            "LOGS_BLOOM" => Ok(HeaderField::LogsBloom),
            "DIFFICULTY" => Ok(HeaderField::Difficulty),
            "NUMBER" => Ok(HeaderField::Number),
            "GAS_LIMIT" => Ok(HeaderField::GasLimit),
            "GAS_USED" => Ok(HeaderField::GasUsed),
            "TIMESTAMP" => Ok(HeaderField::Timestamp),
            "EXTRA_DATA" => Ok(HeaderField::ExtraData),
            "MIX_HASH" => Ok(HeaderField::MixHash),
            "NONCE" => Ok(HeaderField::Nonce),
            "BASE_FEE_PER_GAS" => Ok(HeaderField::BaseFeePerGas),
            _ => Err(()),
        }
    }
}

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
