use crate::{
    datalake::{
        block_sampled::BlockSampledDatalake, datalake_type::DatalakeType,
        envelope::DatalakeEnvelope, transactions::TransactionsInBlockDatalake,
    },
    solidity_types::traits::{Codecs, DatalakeCodecs},
    utils::last_byte_to_u8,
};
use alloy::{
    dyn_abi::{DynSolType, DynSolValue},
    primitives::B256,
};
use anyhow::Result;

pub type BatchedDatalakeEnvelope = Vec<DatalakeEnvelope>;

impl Codecs for BatchedDatalakeEnvelope {
    fn encode(&self) -> Result<Vec<u8>> {
        let mut encoded_datalakes: Vec<DynSolValue> = Vec::new();

        for datalake in self {
            let encoded_datalake = match datalake {
                DatalakeEnvelope::BlockSampled(block_sampled_datalake) => {
                    block_sampled_datalake.encode()?
                }
                DatalakeEnvelope::Transactions(transactions_datalake) => {
                    transactions_datalake.encode()?
                }
            };
            encoded_datalakes.push(DynSolValue::Bytes(encoded_datalake));
        }

        let array_encoded_datalakes = DynSolValue::Array(encoded_datalakes);
        let encoded_datalakes = array_encoded_datalakes.abi_encode();
        Ok(encoded_datalakes)
    }

    fn decode(encoded: &[u8]) -> Result<Vec<DatalakeEnvelope>> {
        let datalakes_type: DynSolType = "bytes[]".parse()?;
        let serialized_datalakes = datalakes_type.abi_decode(encoded)?;
        let mut decoded_datalakes = Vec::new();

        if let Some(datalakes) = serialized_datalakes.as_array() {
            for datalake in datalakes {
                let datalake_as_bytes =
                    datalake.as_bytes().expect("Cannot get bytes from datalake");
                decoded_datalakes.push(DatalakeEnvelope::decode(datalake_as_bytes)?);
            }
        }

        Ok(decoded_datalakes)
    }
}

impl DatalakeCodecs for DatalakeEnvelope {
    fn decode(encoded_datalake: &[u8]) -> Result<Self> {
        let datalake_code = last_byte_to_u8(encoded_datalake.chunks(32).next().unwrap());
        let decoded_datalake = match DatalakeType::from_index(datalake_code)? {
            DatalakeType::BlockSampled => {
                DatalakeEnvelope::BlockSampled(BlockSampledDatalake::decode(encoded_datalake)?)
            }
            DatalakeType::TransactionsInBlock => DatalakeEnvelope::Transactions(
                TransactionsInBlockDatalake::decode(encoded_datalake)?,
            ),
        };
        Ok(decoded_datalake)
    }

    fn encode(&self) -> Result<Vec<u8>> {
        match self {
            DatalakeEnvelope::BlockSampled(datalake) => datalake.encode(),
            DatalakeEnvelope::Transactions(datalake) => datalake.encode(),
        }
    }

    fn get_datalake_type(&self) -> DatalakeType {
        match self {
            DatalakeEnvelope::BlockSampled(_) => DatalakeType::BlockSampled,
            DatalakeEnvelope::Transactions(_) => DatalakeType::TransactionsInBlock,
        }
    }

    fn commit(&self) -> B256 {
        match self {
            DatalakeEnvelope::BlockSampled(datalake) => datalake.commit(),
            DatalakeEnvelope::Transactions(datalake) => datalake.commit(),
        }
    }
}
