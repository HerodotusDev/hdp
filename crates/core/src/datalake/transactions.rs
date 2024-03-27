use std::{
    str::FromStr,
    sync::{Arc, RwLock},
};

use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_primitives::{hex::FromHex, keccak256, Address, U256};
use anyhow::{bail, Result};
use hdp_primitives::{block::transaction::TransactionsCollection, utils::bytes_to_hex_string};
use hdp_provider::evm::AbstractProvider;

use super::{
    base::{DatalakeBase, Derivable},
    Datalake, DatalakeCode, DatalakeCollection,
};

impl DatalakeCollection for TransactionsCollection {
    fn to_index(&self) -> u8 {
        match self {
            TransactionsCollection::Transactions(_) => 0,
            TransactionsCollection::TranasactionReceipts(_) => 1,
        }
    }
}

/// [`TransactionsDatalake`] is a struct that represents a transactions datalake.
///
/// It can represent a transactions datalake for a specific address as sender.
///
/// example 1: from_base_nonce is 0, to_base_nonce is 3 increment is 1
/// target nonce [0, 1, 2, 3]
/// - transaction 1 (nonce 0 -> 1)
/// - transaction 2 (nonce 1 -> 2)
/// - transaction 3 (nonce 2 -> 3)
/// - transaction 4 (nonce 3 -> 4)
///
/// example 2: from_base_nonce is 0, to_base_nonce is 3 increment is 2
/// target nonce [0, 2]
/// - transaction 1 (nonce 0 -> 1)
/// - transaction 2 (nonce 2 -> 3)
///
/// example 3: from_base_nonce is 0, to_base_nonce is 3 increment is 3
/// target nonce [0, 3]
/// - transaction 1 (nonce 0 -> 1)
/// - transaction 2 (nonce 3 -> 4)
///
/// example 4: from_base_nonce is 0, to_base_nonce is 5 increment is 2
/// target nonce [0, 2, 4]
/// - transaction 1 (nonce 0 -> 1)
/// - transaction 2 (nonce 2 -> 3)
/// - transaction 3 (nonce 4 -> 5)
#[derive(Debug, Clone, PartialEq)]
pub struct TransactionsDatalake {
    pub address: Address,
    // start of nonce range the first transaction base on
    pub from_base_nonce: u64,
    // end of nonce range the last transaction base on
    pub to_base_nonce: u64,
    // ex. "tx.to" , "tx.gas_price" or "tx_receipt.success", "tx_receipt.cumulative_gas_used"
    pub sampled_property: TransactionsCollection,
    // increment of nonce range
    pub increment: u64,
}

impl TransactionsDatalake {
    pub fn new(
        address: String,
        from_base_nonce: u64,
        to_base_nonce: u64,
        sampled_property: String,
        increment: u64,
    ) -> Self {
        Self {
            address: Address::from_hex(address).unwrap(),
            from_base_nonce,
            to_base_nonce,
            sampled_property: TransactionsCollection::from_str(&sampled_property).unwrap(),
            increment,
        }
    }

    pub fn get_datalake_code(&self) -> DatalakeCode {
        DatalakeCode::Transactions
    }

    /// Encode the [`TransactionsDatalake`] into a hex string
    pub fn encode(&self) -> Result<String> {
        // Datalake code for transactions datalake is 2
        let datalake_code = DynSolValue::Uint(U256::from(self.get_datalake_code().index()), 256);
        let address = DynSolValue::Address(self.address);
        let from_nonce = DynSolValue::Uint(U256::from(self.from_base_nonce), 256);
        let to_nonce = DynSolValue::Uint(U256::from(self.to_base_nonce), 256);
        let increment = DynSolValue::Uint(U256::from(self.increment), 256);

        let sampled_property =
            DynSolValue::Bytes(self.sampled_property.serialize().unwrap().to_vec());

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
            "(uint256,address,uint256,uint256,uint256,bytes)".parse()?;
        let bytes = Vec::from_hex(encoded).expect("Invalid hex string");
        let decoded = datalake_type.abi_decode_sequence(&bytes)?;

        let value = decoded.as_tuple().unwrap();
        let datalake_code = value[0].as_uint().unwrap().0.to_string().parse::<u8>()?;

        if DatalakeCode::from_index(datalake_code)? != DatalakeCode::Transactions {
            bail!("Encoded datalake is not a transactions datalake");
        }
        let address = value[1].as_address().unwrap();
        let from_base_nonce = value[2].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let to_base_nonce = value[3].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let increment = value[4].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let sampled_property = TransactionsCollection::deserialize(
            &value[5].as_bytes().unwrap().to_vec().try_into().unwrap(),
        )?;

        Ok(Self {
            address,
            from_base_nonce,
            to_base_nonce,
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
    use hdp_primitives::block::transaction::TransactionField;

    use super::*;

    #[test]
    fn test_transactions_datalake() {
        let encoded_datalake= "0x0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000cb96aca8719987d15aecd066b7a1ad5d4d92fdd300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000";

        let transaction_datalake = TransactionsDatalake::new(
            "0xcb96AcA8719987D15aecd066B7a1Ad5D4d92fdD3".to_string(),
            0,
            3,
            "tx.nonce".to_string(),
            1,
        );

        let encoded = transaction_datalake.encode().unwrap();

        assert_eq!(encoded, encoded_datalake);

        assert_eq!(
            transaction_datalake.commit(),
            "0xc8faa3f58f9b18e0716c7c3358f47c3cc5701d71cdf7f4cc93a08afcb4397c5d"
        );

        assert_eq!(
            transaction_datalake.sampled_property,
            TransactionsCollection::Transactions(TransactionField::Nonce)
        );

        let decoded = TransactionsDatalake::decode(encoded).unwrap();
        assert_eq!(decoded, transaction_datalake);
    }
}
