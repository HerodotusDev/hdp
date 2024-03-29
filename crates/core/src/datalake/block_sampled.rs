use std::{str::FromStr, sync::Arc};

use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_primitives::{
    hex::{self, FromHex},
    keccak256, Address, U256,
};
use anyhow::{bail, Result};
use hdp_primitives::{
    block::{account::AccountField, header::HeaderField},
    utils::bytes_to_hex_string,
};
use hdp_provider::evm::AbstractProvider;
use tokio::sync::RwLock;

use crate::compiler::block_sampled::compile_block_sampled_datalake;

use super::{
    base::{DatalakeBase, DatalakeResult, Derivable},
    Datalake, DatalakeCollection,
};

#[derive(Debug, PartialEq)]
pub enum BlockSampledCollection {
    Header,
    Account,
    Storage,
}

impl DatalakeCollection for BlockSampledCollection {
    fn to_index(&self) -> u8 {
        match self {
            BlockSampledCollection::Header => 1,
            BlockSampledCollection::Account => 2,
            BlockSampledCollection::Storage => 3,
        }
    }
}

/// BlockSampledDatalake represents a datalake for a block range
#[derive(Debug, Clone, PartialEq)]
pub struct BlockSampledDatalake {
    pub block_range_start: u64,
    pub block_range_end: u64,
    pub sampled_property: String,
    pub increment: u64,
}

impl BlockSampledDatalake {
    pub fn new(
        block_range_start: u64,
        block_range_end: u64,
        sampled_property: String,
        increment: u64,
    ) -> Self {
        Self {
            block_range_start,
            block_range_end,
            sampled_property,
            increment,
        }
    }

    /// Get `header`, `account` or `storage` type of the block sampled datalake
    pub fn get_collection_type(&self) -> BlockSampledCollection {
        match serialize_sampled_property(&self.sampled_property).unwrap()[0] {
            1 => BlockSampledCollection::Header,
            2 => BlockSampledCollection::Account,
            3 => BlockSampledCollection::Storage,
            _ => panic!("Invalid collection type"),
        }
    }

    /// Encode the block sampled datalake
    pub fn encode(&self) -> Result<String> {
        let block_range_start = DynSolValue::Uint(U256::from(self.block_range_start), 256);
        let block_range_end = DynSolValue::Uint(U256::from(self.block_range_end), 256);
        let sampled_property =
            DynSolValue::Bytes(serialize_sampled_property(&self.sampled_property)?);
        let increment = DynSolValue::Uint(U256::from(self.increment), 256);
        let datalake_code = DynSolValue::Uint(U256::from(0), 256);

        let tuple_value = DynSolValue::Tuple(vec![
            datalake_code,
            block_range_start,
            block_range_end,
            increment,
            sampled_property,
        ]);

        match tuple_value.abi_encode_sequence() {
            Some(encoded_datalake) => Ok(bytes_to_hex_string(&encoded_datalake)),
            None => bail!("Encoding failed"),
        }
    }

    /// Get the commitment hash of the block sampled datalake
    pub fn commit(&self) -> String {
        let encoded_datalake = self.encode().expect("Encoding failed");
        let bytes = Vec::from_hex(encoded_datalake).expect("Invalid hex string");
        let hash = keccak256(bytes);
        format!("0x{:x}", hash)
    }

    /// Decode the encoded block sampled datalake
    pub fn decode(encoded: String) -> Result<Self> {
        let datalake_type: DynSolType = "(uint256,uint256,uint256,uint256,bytes)".parse()?;
        let bytes = Vec::from_hex(encoded).expect("Invalid hex string");
        let decoded = datalake_type.abi_decode_sequence(&bytes)?;

        let value = decoded.as_tuple().unwrap();
        let datalake_code = value[0].as_uint().unwrap().0.to_string().parse::<u64>()?;

        if datalake_code != 0 {
            bail!("Encoded datalake is not a block sample datalake");
        }

        let block_range_start = value[1].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let block_range_end = value[2].as_uint().unwrap().0.to_string().parse::<u64>()?;
        let sampled_property = deserialize_sampled_property(value[4].as_bytes().unwrap())?;
        let increment = value[3].as_uint().unwrap().0.to_string().parse::<u64>()?;

        Ok(Self {
            block_range_start,
            block_range_end,
            sampled_property,
            increment,
        })
    }

    pub async fn compile(
        &self,
        provider: &Arc<RwLock<AbstractProvider>>,
    ) -> Result<DatalakeResult> {
        compile_block_sampled_datalake(
            self.block_range_start,
            self.block_range_end,
            &self.sampled_property,
            self.increment,
            provider,
        )
        .await
    }
}

impl Default for BlockSampledDatalake {
    fn default() -> Self {
        Self::new(0, 0, "".to_string(), 0)
    }
}

impl Derivable for BlockSampledDatalake {
    fn derive(&self) -> DatalakeBase {
        DatalakeBase::new(&self.commit(), Datalake::BlockSampled(self.clone()))
    }
}

pub fn serialize_sampled_property(sampled_property: &str) -> Result<Vec<u8>> {
    let tokens: Vec<&str> = sampled_property.split('.').collect();
    let collection = match tokens[0] {
        "header" => BlockSampledCollection::Header,
        "account" => BlockSampledCollection::Account,
        "storage" => BlockSampledCollection::Storage,
        _ => bail!("Unknown collection type"),
    };

    let mut serialized = Vec::new();
    serialized.push(match collection {
        BlockSampledCollection::Header => 1,
        BlockSampledCollection::Account => 2,
        BlockSampledCollection::Storage => 3,
    });

    match collection {
        BlockSampledCollection::Header => {
            let index = HeaderField::from_str(tokens[1].to_uppercase().as_str())?.to_index();
            serialized.push(index);
        }
        BlockSampledCollection::Account | BlockSampledCollection::Storage => {
            // if !is_address(tokens[1]) {
            //     panic!("Invalid account address");
            // }
            let account_bytes = hex::decode(&tokens[1][2..]).expect("Account decoding failed");
            serialized.extend_from_slice(&account_bytes);

            if collection == BlockSampledCollection::Account {
                serialized
                    .push(AccountField::from_str(tokens[2].to_uppercase().as_str())?.to_index());
            } else {
                if tokens[2].len() != 66 || !tokens[2][2..].chars().all(|c| c.is_ascii_hexdigit()) {
                    bail!("Invalid storage slot");
                }
                let slot_bytes =
                    hex::decode(&tokens[2][2..]).expect("Storage slot decoding failed");
                serialized.extend_from_slice(&slot_bytes);
            }
        }
    }

    Ok(serialized)
}

fn deserialize_sampled_property(serialized: &[u8]) -> Result<String> {
    let property_type = serialized[0];
    let property = ["header", "account", "storage"][property_type as usize - 1];

    match property {
        "header" => {
            let header_prop_index = serialized[1];
            let sub_property_type = match HeaderField::from_index(header_prop_index) {
                Some(field) => field.as_str(),
                None => bail!("Invalid header property index"),
            };
            Ok(format!("{}.{}", property, sub_property_type.to_lowercase()))
        }
        "account" => {
            let account = Address::from_slice(&serialized[1..21]);
            let account_checksum = format!("{:?}", account);
            let account_prop_index = serialized[21];
            let sub_property_type = match AccountField::from_index(account_prop_index) {
                Some(field) => field.as_str(),
                None => bail!("Invalid account property index"),
            };
            Ok(format!(
                "{}.{}.{}",
                property,
                account_checksum,
                sub_property_type.to_lowercase()
            ))
        }
        "storage" => {
            let account = Address::from_slice(&serialized[1..21]);
            let account_checksum = format!("{:?}", account);
            let slot = &serialized[21..53];
            let slot_string = bytes_to_hex_string(slot);

            Ok(format!("{}.{}.{}", property, account_checksum, slot_string))
        }
        _ => bail!("Invalid collection id"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_datalake_for_header() {
        let encoded_block_sample_datalake = "0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009eb0f600000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f000000000000000000000000000000000000000000000000000000000000";
        let decoded_datalake =
            BlockSampledDatalake::decode(encoded_block_sample_datalake.to_string()).unwrap();
        let block_datalake =
            BlockSampledDatalake::new(10399990, 10400000, "header.base_fee_per_gas".to_string(), 1);
        assert_eq!(
            decoded_datalake.encode().unwrap(),
            block_datalake.encode().unwrap()
        );

        assert_eq!(
            decoded_datalake.encode().unwrap(),
            encoded_block_sample_datalake
        );

        assert_eq!(
            block_datalake.commit(),
            "0x26365cf5692cc38bca06023b8b62ceb0f6bd959a57e3c453be213d1b71d73732".to_string()
        );

        assert_eq!(
            block_datalake.get_collection_type(),
            BlockSampledCollection::Header
        );
    }

    #[test]
    fn test_block_datalake_for_header_massive() {
        let encoded_block_sample_datalake = "0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009d2a6000000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f000000000000000000000000000000000000000000000000000000000000";
        let decoded_datalake: BlockSampledDatalake =
            BlockSampledDatalake::decode(encoded_block_sample_datalake.to_string()).unwrap();
        let block_datalake =
            BlockSampledDatalake::new(10300000, 10400000, "header.base_fee_per_gas".to_string(), 1);

        assert_eq!(
            decoded_datalake.encode().unwrap(),
            block_datalake.encode().unwrap()
        );

        assert_eq!(
            decoded_datalake.encode().unwrap(),
            encoded_block_sample_datalake
        );

        assert_eq!(
            block_datalake.commit(),
            "0xc21f3b3a49c5bed8b7624d0efc050a2a481f06f627d04212bf1d745d0aa5c6f1".to_string()
        );

        assert_eq!(
            block_datalake.get_collection_type(),
            BlockSampledCollection::Header
        );
    }

    #[test]
    fn test_block_datalake_for_account() {
        let encoded_block_sample_datalake = "0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009eb0f600000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000016027b2f05ce9ae365c3dbf30657e2dc6449989e83d60000000000000000000000";
        let decoded_datalake =
            BlockSampledDatalake::decode(encoded_block_sample_datalake.to_string()).unwrap();
        let block_datalake = BlockSampledDatalake::new(
            10399990,
            10400000,
            "account.0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6.nonce".to_string(),
            1,
        );
        assert_eq!(decoded_datalake, block_datalake);
        assert_eq!(
            decoded_datalake.encode().unwrap(),
            encoded_block_sample_datalake
        );

        assert_eq!(
            block_datalake.commit(),
            "0x79b0d86f9b08c78f527666d4d39d01349530ced0a3d37f4c63e7108814a670b7".to_string()
        );

        assert_eq!(
            block_datalake.get_collection_type(),
            BlockSampledCollection::Account
        );
    }

    #[test]
    fn test_block_datalake_for_account_2() {
        let encoded_block_sample_datalake = "0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004b902400000000000000000000000000000000000000000000000000000000004b9027000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000016020a4de450feb156a2a51ed159b2fb99da26e5f3a30000000000000000000000";
        let decoded_datalake =
            BlockSampledDatalake::decode(encoded_block_sample_datalake.to_string()).unwrap();
        let block_datalake = BlockSampledDatalake::new(
            4952100,
            4952103,
            "account.0x0a4de450feb156a2a51ed159b2fb99da26e5f3a3.nonce".to_string(),
            1,
        );
        let serialized = block_datalake.encode().unwrap();
        assert_eq!(serialized, encoded_block_sample_datalake);
        assert_eq!(decoded_datalake, block_datalake);
        assert_eq!(
            decoded_datalake.encode().unwrap(),
            encoded_block_sample_datalake
        );

        assert_eq!(
            block_datalake.commit(),
            "0x6db54c04174bd625449785ca58efd313e016b807d0a17add522d74e0e27c3b08".to_string()
        );

        assert_eq!(
            block_datalake.get_collection_type(),
            BlockSampledCollection::Account
        );
    }

    #[test]
    fn test_block_datalake_for_storage() {
        let encoded_block_sample_datalake = "0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009eb0f600000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000035037b2f05ce9ae365c3dbf30657e2dc6449989e83d600000000000000000000000000000000000000000000000000000000000000ff0000000000000000000000";
        let decoded_datalake =
            BlockSampledDatalake::decode(encoded_block_sample_datalake.to_string()).unwrap();
        let block_datalake = BlockSampledDatalake::new(
            10399990,
            10400000,
            "storage.0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6.0x00000000000000000000000000000000000000000000000000000000000000ff".to_string(),
            1,
        );
        assert_eq!(decoded_datalake, block_datalake);
        assert_eq!(
            decoded_datalake.encode().unwrap(),
            encoded_block_sample_datalake
        );

        assert_eq!(
            block_datalake.commit(),
            "0x147dc75fd577a75dca31c0c5181539a1078c48759e379685b827f8c0e3f0b6ef".to_string()
        );

        assert_eq!(
            block_datalake.get_collection_type(),
            BlockSampledCollection::Storage
        );
    }

    #[test]
    fn test_sampled_property() {
        let target = "header.base_fee_per_gas";
        let serialized = serialize_sampled_property(target).unwrap();
        let property = deserialize_sampled_property(&serialized).unwrap();
        assert_eq!(property, target);

        let target = "account.0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6.nonce";
        let serialized = serialize_sampled_property(target).unwrap();
        let property = deserialize_sampled_property(&serialized).unwrap();
        assert_eq!(property, target);

        let target = "storage.0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6.0x00000000000000000000000000000000000000000000000000000000000000ff";
        let serialized = serialize_sampled_property(target).unwrap();
        let property = deserialize_sampled_property(&serialized).unwrap();
        assert_eq!(property, target);
    }
}
