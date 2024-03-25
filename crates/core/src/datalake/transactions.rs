use std::sync::{Arc, RwLock};

use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_primitives::{hex::FromHex, keccak256, Address, U256};
use anyhow::{bail, Result};
use hdp_primitives::{block::transaction::TransactionField, utils::bytes_to_hex_string};
use hdp_provider::evm::AbstractProvider;

use super::{
    base::{DatalakeBase, Derivable},
    Datalake, DatalakeCode,
};

/// [`TransactionsDatalake`] is a struct that represents a transactions datalake.
///
/// It can represent a transactions datalake for a specific address as either sender or receiver.
#[derive(Debug, Clone, PartialEq)]
pub struct TransactionsDatalake {
    pub account_type: AccountType,
    pub address: Address,
    pub from_nonce: u64,
    pub to_nonce: u64,
    // TODO: In the end should be contain transaction receipt also
    pub sampled_property: TransactionField,
    pub increment: u64,
}

/// [`TransactionsDatalake`] have two types of accounts: sender and receiver.
#[derive(Debug, Clone, PartialEq)]
pub enum AccountType {
    Sender = 0,
    Receiver = 1,
}

impl AccountType {
    pub fn index(&self) -> usize {
        match self {
            AccountType::Sender => 0,
            AccountType::Receiver => 1,
        }
    }
}

impl TransactionsDatalake {
    pub fn new(
        account_type: AccountType,
        address: Address,
        from_nonce: u64,
        to_nonce: u64,
        sampled_property: TransactionField,
        increment: u64,
    ) -> Self {
        Self {
            account_type,
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

    /// Encode the [`TransactionsDatalake`] into a hex string
    pub fn encode(&self) -> Result<String> {
        // Datalake code for transactions datalake is 2
        let datalake_code = DynSolValue::Uint(U256::from(self.get_datalake_code().index()), 256);
        let account_type = DynSolValue::Uint(U256::from(self.account_type.index()), 256);
        let address = DynSolValue::Address(self.address);
        let from_nonce = DynSolValue::Uint(U256::from(self.from_nonce), 256);
        let to_nonce = DynSolValue::Uint(U256::from(self.to_nonce), 256);
        let increment = DynSolValue::Uint(U256::from(self.increment), 256);
        let sampled_property = DynSolValue::Uint(U256::from(self.sampled_property.to_index()), 256);

        let tuple_value = DynSolValue::Tuple(vec![
            datalake_code,
            account_type,
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
            "(uint256,uint256,address,uint256,uint256,uint256,uint256)".parse()?;
        let bytes = Vec::from_hex(encoded).expect("Invalid hex string");
        let decoded = datalake_type.abi_decode_sequence(&bytes)?;

        let value = decoded.as_tuple().unwrap();
        let datalake_code = value[0].as_uint().unwrap().0.to_string().parse::<u8>()?;

        if DatalakeCode::from_index(datalake_code)? != DatalakeCode::Transactions {
            bail!("Encoded datalake is not a transactions datalake");
        }

        let account_type = match value[1].as_uint().unwrap().0.to_string().parse::<u64>()? {
            0 => AccountType::Sender,
            1 => AccountType::Receiver,
            _ => bail!("Invalid account type"),
        };
        let address = value[2].as_address().unwrap();
        let from_nonce = value[3].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let to_nonce = value[4].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let increment = value[5].as_uint().unwrap().0.to_string().parse::<u64>()?;

        let sampled_property =
            TransactionField::from_index(value[6].as_uint().unwrap().0.to_string().parse::<u8>()?)
                .unwrap();

        Ok(Self {
            account_type,
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
