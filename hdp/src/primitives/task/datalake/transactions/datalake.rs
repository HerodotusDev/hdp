//![`TransactionsInBlockDatalake`] is a struct that represents a transactions datalake.
//!
//! It can represent all transactions in a block with a specific increment.
//!
//! Example: `TransactionsInBlockDatalake { target_block: 100, sampled_property: "tx.to", increment: 1 }`
//! represents all transactions in block 100 with a `tx.to` property sampled with an increment of 1.

use std::num::ParseIntError;
use std::str::FromStr;

use alloy::consensus::TxType;
use alloy::primitives::U256;
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::primitives::{task::datalake::envelope::default_increment, ChainId};

use super::TransactionsCollection;

/// [`TransactionsInBlockDatalake`] is a struct that represents a transactions datalake.
/// It contains chain id, target block, transaction range, sampled property, and other properties.
///
/// Transaction range: [start_index..end_index] with specified increment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionsInBlockDatalake {
    /// Chain id of the datalake
    pub chain_id: ChainId,
    /// Target block number
    pub target_block: u64,
    /// Start index of transactions range (default 0)
    pub start_index: u64,
    /// End index of transactions range, not included in the range (default last)
    pub end_index: u64,
    /// Increment of transactions, Defaults to 1 if not present.
    #[serde(default = "default_increment")]
    pub increment: u64,
    /// Filter out the specific type of Txs
    pub included_types: IncludedTypes,
    /// Sampled property (e.g., "tx.to", "tx.gas_price", "tx_receipt.success", "tx_receipt.cumulative_gas_used")
    pub sampled_property: TransactionsCollection,
}

impl TransactionsInBlockDatalake {
    pub fn new(
        chain_id: ChainId,
        target_block: u64,
        sampled_property: TransactionsCollection,
        start_index: u64,
        end_index: u64,
        increment: u64,
        included_types: IncludedTypes,
    ) -> Self {
        Self {
            chain_id,
            target_block,
            sampled_property,
            start_index,
            end_index,
            increment,
            included_types,
        }
    }
}

/// A struct to represent the included types in a transactions datalake
/// The included types are represented as a 4 byte array
/// Each byte represents a type of transaction to be included in the datalake
/// The bytes are represented as follows:
/// 0: Legacy
/// 1: EIP-2930
/// 2: EIP-1559
/// 3: EIP-4844
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncludedTypes {
    legacy: bool,
    eip2930: bool,
    eip1559: bool,
    eip4844: bool,
}

impl IncludedTypes {
    /// All transaction types(Legacy, EIP-1559, EIP-2930, EIP-4844) are included
    pub const ALL: Self = Self {
        legacy: true,
        eip1559: true,
        eip2930: true,
        eip4844: true,
    };

    pub fn to_be_bytes(&self) -> [u8; 4] {
        let mut bytes = [0; 4];
        if self.legacy {
            bytes[0] = 1;
        }
        if self.eip2930 {
            bytes[1] = 1;
        }
        if self.eip1559 {
            bytes[2] = 1;
        }
        if self.eip4844 {
            bytes[3] = 1;
        }
        bytes
    }

    pub fn from_be_bytes(bytes: [u8; 4]) -> Self {
        let mut included_types = IncludedTypes {
            legacy: false,
            eip2930: false,
            eip1559: false,
            eip4844: false,
        };
        if bytes[0] == 1 {
            included_types.legacy = true;
        }
        if bytes[1] == 1 {
            included_types.eip2930 = true;
        }
        if bytes[2] == 1 {
            included_types.eip1559 = true;
        }
        if bytes[3] == 1 {
            included_types.eip4844 = true;
        }
        included_types
    }

    /// Converts a slice of bytes into an [`IncludedTypes`] instance.
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - The input slice is not exactly 4 bytes long.
    /// - Any byte in the slice is not 0 or 1.
    /// - All bytes in the slice are 0 (i.e., no transaction type is included).
    pub fn from_bytes(included_types: &[u8]) -> Self {
        if included_types.len() != 4 {
            panic!("Included types must be 4 bytes long");
        }
        if included_types.iter().any(|&x| x > 1) {
            panic!("Included types must be either 0 or 1");
        }
        if included_types.iter().all(|&x| x == 0) {
            panic!("At least one type must be included");
        }
        let mut inner = [0; 4];
        inner.copy_from_slice(included_types);
        Self::from_be_bytes(inner)
    }

    pub fn is_included(&self, target_type: TxType) -> bool {
        // check with the index of bytes is either 0 or 1
        let inner_bytes = self.to_be_bytes();
        inner_bytes[target_type as usize] != 0
    }
}

impl From<IncludedTypes> for U256 {
    fn from(value: IncludedTypes) -> U256 {
        let mut bytes = [0; 32];
        let inner_bytes = value.to_be_bytes();
        bytes[28..32].copy_from_slice(&inner_bytes);
        U256::from_be_bytes(bytes)
    }
}

// Implementation of From<U256> for Self
impl From<U256> for IncludedTypes {
    fn from(value: U256) -> IncludedTypes {
        let bytes: [u8; 32] = value.to_be_bytes();
        let mut inner = [0; 4];
        inner.copy_from_slice(&bytes[28..32]);
        IncludedTypes::from_be_bytes(inner)
    }
}

impl FromStr for IncludedTypes {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let included_types: Vec<u8> = s
            .split(',')
            .map(|x| x.parse())
            .collect::<Result<Vec<_>, _>>()?;

        if included_types.len() != 4 {
            panic!("Included types must be 4 bytes long");
        }

        Ok(IncludedTypes::from_bytes(&included_types))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_included_types() {
        let included_types = IncludedTypes::ALL;
        assert!(included_types.is_included(TxType::Legacy));
        assert!(included_types.is_included(TxType::Eip2930));
        assert!(included_types.is_included(TxType::Eip1559));
        assert!(included_types.is_included(TxType::Eip4844));

        let uint256: U256 = included_types.into();
        assert_eq!(uint256, U256::from(0x01010101));

        let included_types = IncludedTypes::from(uint256);
        assert!(included_types.is_included(TxType::Legacy));
        assert!(included_types.is_included(TxType::Eip2930));
        assert!(included_types.is_included(TxType::Eip1559));
        assert!(included_types.is_included(TxType::Eip4844));
    }

    #[test]
    fn test_included_types_partial() {
        let included_types = IncludedTypes::from_bytes(&[1, 0, 1, 0]);
        assert!(included_types.is_included(TxType::Legacy));
        assert!(!included_types.is_included(TxType::Eip2930));
        assert!(included_types.is_included(TxType::Eip1559));
        assert!(!included_types.is_included(TxType::Eip4844));

        let uint256: U256 = included_types.into();
        assert_eq!(uint256, U256::from(0x01000100));

        let included_types = IncludedTypes::from(uint256);
        assert!(included_types.is_included(TxType::Legacy));
        assert!(!included_types.is_included(TxType::Eip2930));
        assert!(included_types.is_included(TxType::Eip1559));
        assert!(!included_types.is_included(TxType::Eip4844));
    }
}
