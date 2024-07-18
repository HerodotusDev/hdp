use crate::solidity_types::traits::DatalakeCodecs;
use crate::task::datalake::block_sampled::{BlockSampledCollection, BlockSampledDatalake};
use crate::task::datalake::datalake_type::DatalakeType;
use crate::task::datalake::DatalakeCollection;

use alloy::primitives::keccak256;
use alloy::{
    dyn_abi::{DynSolType, DynSolValue},
    primitives::B256,
};
use anyhow::{bail, Result};

impl DatalakeCodecs for BlockSampledDatalake {
    /// Get the datalake code for block sampled datalake
    fn get_datalake_type(&self) -> DatalakeType {
        DatalakeType::BlockSampled
    }

    /// Encode the block sampled datalake
    fn encode(&self) -> Result<Vec<u8>> {
        let datalake_code: DynSolValue = self.get_datalake_type().to_u8().into();
        let chain_id: DynSolValue = self.chain_id.into();
        let block_range_start: DynSolValue = self.block_range_start.into();
        let block_range_end: DynSolValue = self.block_range_end.into();
        let sampled_property: DynSolValue = self.sampled_property.serialize()?.into();
        let increment: DynSolValue = self.increment.into();

        let tuple_value = DynSolValue::Tuple(vec![
            datalake_code,
            chain_id,
            block_range_start,
            block_range_end,
            increment,
            sampled_property,
        ]);

        match tuple_value.abi_encode_sequence() {
            Some(encoded_datalake) => Ok(encoded_datalake),
            None => bail!("Encoding failed"),
        }
    }

    /// Get the commitment hash of the block sampled datalake
    fn commit(&self) -> B256 {
        keccak256(self.encode().expect("Encoding failed"))
    }

    /// Decode the encoded block sampled datalake
    fn decode(encoded: &[u8]) -> Result<Self> {
        let abi_type: DynSolType = "(uint256,uint256,uint256,uint256,uint256,bytes)".parse()?;
        let decoded = abi_type.abi_decode_sequence(encoded)?;
        let value = decoded.as_tuple().unwrap();
        let datalake_code = value[0].as_uint().unwrap().0.to_string().parse::<u8>()?;

        if DatalakeType::from_index(datalake_code)? != DatalakeType::BlockSampled {
            bail!("Encoded datalake is not a block sample datalake");
        }

        let chain_id = value[1].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let block_range_start = value[2].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let block_range_end = value[3].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let increment = value[4].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let sampled_property = BlockSampledCollection::deserialize(value[5].as_bytes().unwrap())?;

        Ok(Self {
            chain_id,
            block_range_start,
            block_range_end,
            increment,
            sampled_property,
        })
    }
}
