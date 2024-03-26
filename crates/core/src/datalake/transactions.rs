use std::sync::{Arc, RwLock};

use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_primitives::{hex::FromHex, keccak256, Address, U256};
use anyhow::{bail, Result};
use hdp_primitives::{
    block::transaction::{TransactionDatalakeField, TransactionsCollection},
    utils::bytes_to_hex_string,
};
use hdp_provider::evm::AbstractProvider;

use super::{
    base::{DatalakeBase, Derivable},
    Datalake, DatalakeCode, DatalakeCollection,
};

impl DatalakeCollection for TransactionsCollection {
    fn to_index(&self) -> u8 {
        match self {
            TransactionsCollection::TransactionsBySender => 0,
            TransactionsCollection::TranasactionReceiptsBySender => 1,
        }
    }

    fn from_index(index: u8) -> Result<Self> {
        match index {
            0 => Ok(TransactionsCollection::TransactionsBySender),
            1 => Ok(TransactionsCollection::TranasactionReceiptsBySender),
            _ => bail!("Invalid transactions collection index"),
        }
    }
}

/// [`TransactionsDatalake`] is a struct that represents a transactions datalake.
///
/// It can represent a transactions datalake for a specific address as sender.
///
/// example 1: from_nonce is 0, to_nonce is 3 increment is 1
/// target nonce [0, 1, 2, 3]
/// - transaction 1 (nonce 0 -> 1)
/// - transaction 2 (nonce 1 -> 2)
/// - transaction 3 (nonce 2 -> 3)
/// - transaction 4 (nonce 3 -> 4)
///
/// example 2: from_nonce is 0, to_nonce is 3 increment is 2
/// target nonce [0, 2]
/// - transaction 1 (nonce 0 -> 1)
/// - transaction 2 (nonce 2 -> 3)
///
/// example 3: from_nonce is 0, to_nonce is 3 increment is 3
/// target nonce [0, 3]
/// - transaction 1 (nonce 0 -> 1)
/// - transaction 2 (nonce 3 -> 4)
///
/// example 4: from_nonce is 0, to_nonce is 5 increment is 2
/// target nonce [0, 2, 4]
/// - transaction 1 (nonce 0 -> 1)
/// - transaction 2 (nonce 2 -> 3)
/// - transaction 3 (nonce 4 -> 5)
#[derive(Debug, Clone, PartialEq)]
pub struct TransactionsDatalake {
    pub address: Address,
    pub from_nonce: u64,
    pub to_nonce: u64,
    pub sampled_property: TransactionDatalakeField,
    pub increment: u64,
}

impl TransactionsDatalake {
    pub fn new(
        address: Address,
        from_nonce: u64,
        to_nonce: u64,
        sampled_property: TransactionDatalakeField,
        increment: u64,
    ) -> Self {
        Self {
            address,
            from_nonce,
            to_nonce,
            sampled_property,
            increment,
        }
    }

    pub fn get_datalake_code(&self) -> DatalakeCode {
        DatalakeCode::Transactions
    }

    pub fn get_collection_type(&self) -> TransactionsCollection {
        self.sampled_property.parse_collection()
    }

    /// Encode the [`TransactionsDatalake`] into a hex string
    pub fn encode(&self) -> Result<String> {
        // Datalake code for transactions datalake is 2
        let datalake_code = DynSolValue::Uint(U256::from(self.get_datalake_code().index()), 256);
        let address = DynSolValue::Address(self.address);
        let from_nonce = DynSolValue::Uint(U256::from(self.from_nonce), 256);
        let to_nonce = DynSolValue::Uint(U256::from(self.to_nonce), 256);
        let increment = DynSolValue::Uint(U256::from(self.increment), 256);
        let sampled_property = DynSolValue::Uint(U256::from(self.sampled_property.to_index()), 256);

        let tuple_value = DynSolValue::Tuple(vec![
            datalake_code,
            address,
            from_nonce,
            to_nonce,
            increment,
            sampled_property,
        ]);

        match tuple_value.abi_encode_sequence() {
            Some(encoded_datalake) => Ok(bytes_to_hex_string(&encoded_datalake)),
            None => bail!("Encoding failed"),
        }
    }

    /// Get the commitment hash of the [`TransactionsDatalake`]
    pub fn commit(&self) -> String {
        let encoded_datalake = self.encode().expect("Encoding failed");
        let bytes = Vec::from_hex(encoded_datalake).expect("Invalid hex string");
        let hash = keccak256(bytes);
        format!("0x{:x}", hash)
    }

    /// Decode the encoded transactions datalake hex string into a [`TransactionsDatalake`]
    pub fn decode(encoded: String) -> Result<Self> {
        let datalake_type: DynSolType =
            "(uint256,address,uint256,uint256,uint256,uint256)".parse()?;
        let bytes = Vec::from_hex(encoded).expect("Invalid hex string");
        let decoded = datalake_type.abi_decode_sequence(&bytes)?;

        let value = decoded.as_tuple().unwrap();
        let datalake_code = value[0].as_uint().unwrap().0.to_string().parse::<u8>()?;

        if DatalakeCode::from_index(datalake_code)? != DatalakeCode::Transactions {
            bail!("Encoded datalake is not a transactions datalake");
        }
        let address = value[1].as_address().unwrap();
        let from_nonce = value[2].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let to_nonce = value[3].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let increment = value[4].as_uint().unwrap().0.to_string().parse::<u64>()?;

        let sampled_property = TransactionDatalakeField::from_index(
            value[5].as_uint().unwrap().0.to_string().parse::<u8>()?,
        )
        .unwrap();

        Ok(Self {
            address,
            from_nonce,
            to_nonce,
            sampled_property,
            increment,
        })
    }

    pub async fn compile(&self, _: &Arc<RwLock<AbstractProvider>>) -> Result<()> {
        // TODO: Implement compilation
        Ok(())
    }
}

impl Derivable for TransactionsDatalake {
    fn derive(&self) -> DatalakeBase {
        DatalakeBase::new(&self.commit(), Datalake::Transactions(self.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hdp_primitives::block::transaction::TransactionDatalakeField;

    #[test]
    fn test_transactions_datalake() {
        let encoded_datalake= "0x0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000cb96aca8719987d15aecd066b7a1ad5d4d92fdd30000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001";

        let transaction_datalake = TransactionsDatalake::new(
            Address::from_hex("0xcb96AcA8719987D15aecd066B7a1Ad5D4d92fdD3").unwrap(),
            0,
            3,
            TransactionDatalakeField::Nonce,
            1,
        );

        let encoded = transaction_datalake.encode().unwrap();

        assert_eq!(encoded, encoded_datalake);

        assert_eq!(
            transaction_datalake.commit(),
            "0x7146e0abe3c81792b754792f40cb3668b4ba057904938dd8bf781e18ed182c05"
        );

        assert_eq!(
            transaction_datalake.get_collection_type(),
            TransactionsCollection::TransactionsBySender
        );
    }
}
