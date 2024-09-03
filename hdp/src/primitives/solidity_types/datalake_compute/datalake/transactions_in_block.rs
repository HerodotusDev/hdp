use crate::primitives::{
    solidity_types::traits::DatalakeCodecs,
    task::datalake::{
        datalake_type::DatalakeType,
        transactions::{IncludedTypes, TransactionsCollection, TransactionsInBlockDatalake},
        DatalakeCollection,
    },
    ChainId,
};
use alloy::primitives::keccak256;
use alloy::{
    dyn_abi::{DynSolType, DynSolValue},
    primitives::B256,
};
use anyhow::{bail, Result};

impl DatalakeCodecs for TransactionsInBlockDatalake {
    /// Get the datalake code for transactions datalake
    fn get_datalake_type(&self) -> DatalakeType {
        DatalakeType::TransactionsInBlock
    }

    /// Encode the [`TransactionsInBlockDatalake`] into a hex string
    fn encode(&self) -> Result<Vec<u8>> {
        let datalake_code: DynSolValue = self.get_datalake_type().to_u8().into();
        let chain_id: DynSolValue = self.chain_id.to_numeric_id().into();
        let target_block: DynSolValue = self.target_block.into();
        let sampled_property: DynSolValue = self.sampled_property.serialize()?.into();
        let start_index: DynSolValue = self.start_index.into();
        let end_index: DynSolValue = self.end_index.into();
        let increment: DynSolValue = self.increment.into();
        let included_types: DynSolValue = self.included_types.to_uint256().into();

        let tuple_value = DynSolValue::Tuple(vec![
            datalake_code,
            chain_id,
            target_block,
            start_index,
            end_index,
            increment,
            included_types,
            sampled_property,
        ]);

        match tuple_value.abi_encode_sequence() {
            Some(encoded_datalake) => Ok(encoded_datalake),
            None => bail!("Encoding failed"),
        }
    }

    /// Get the commitment hash of the [`TransactionsDatalake`]
    fn commit(&self) -> B256 {
        let encoded_datalake = self.encode().expect("Encoding failed");
        keccak256(encoded_datalake)
    }

    /// Decode the encoded transactions datalake hex string into a [`TransactionsDatalake`]
    fn decode(encoded: &[u8]) -> Result<Self> {
        let abi_type: DynSolType =
            "(uint256,uint256, uint256, uint256, uint256, uint256, uint256, bytes)".parse()?;
        let decoded = abi_type.abi_decode_sequence(encoded)?;

        let value = decoded.as_tuple().unwrap();
        let datalake_code = value[0].as_uint().unwrap().0.to_string().parse::<u8>()?;

        if DatalakeType::from_index(datalake_code)? != DatalakeType::TransactionsInBlock {
            bail!("Encoded datalake is not a transactions datalake");
        }

        let chain_id =
            ChainId::from_numeric_id(value[1].as_uint().unwrap().0.to_string().parse::<u128>()?)?;
        let target_block = value[2].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let start_index = value[3].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let end_index = value[4].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let increment = value[5].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let included_types = IncludedTypes::from_uint256(value[6].as_uint().unwrap().0);
        let sampled_property = TransactionsCollection::deserialize(value[7].as_bytes().unwrap())?;

        Ok(Self {
            chain_id,
            target_block,
            start_index,
            end_index,
            increment,
            included_types,
            sampled_property,
        })
    }
}
