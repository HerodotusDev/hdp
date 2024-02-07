use std::{str::FromStr, sync::Arc};

use crate::{
    block::{account::AccountField, header::HeaderField, Collection},
    fetcher::AbstractFetcher,
    utils::bytes_to_hex_string,
};
use alloy_dyn_abi::{DynSolType, DynSolValue};
use alloy_primitives::{
    hex::{self, FromHex},
    keccak256, Address, U256,
};
use anyhow::{bail, Result};
use tokio::sync::RwLock;

use crate::compiler::block_sampled::compile_block_sampled_datalake;

use super::{
    base::{DatalakeBase, Derivable},
    Datalake,
};

/// BlockSampledDatalake represents a datalake for a block range
#[derive(Debug, Clone, PartialEq)]
pub struct BlockSampledDatalake {
    pub block_range_start: usize,
    pub block_range_end: usize,
    pub sampled_property: String,
    pub increment: usize,
}

impl ToString for BlockSampledDatalake {
    fn to_string(&self) -> String {
        let block_range_start = DynSolValue::Uint(U256::from(self.block_range_start), 256);
        let block_range_end = DynSolValue::Uint(U256::from(self.block_range_end), 256);
        let sampled_property =
            DynSolValue::Bytes(serialize_sampled_property(&self.sampled_property));
        let increment = DynSolValue::Uint(U256::from(self.increment), 256);
        let tuple_value = DynSolValue::Tuple(vec![
            block_range_start,
            block_range_end,
            increment,
            sampled_property,
        ]);
        let encoded_datalake = tuple_value.abi_encode();
        let hash = keccak256(encoded_datalake);
        format!("0x{:x}", hash)
    }
}

impl BlockSampledDatalake {
    pub fn new(
        block_range_start: usize,
        block_range_end: usize,
        sampled_property: String,
        increment: usize,
    ) -> Self {
        Self {
            block_range_start,
            block_range_end,
            sampled_property,
            increment,
        }
    }

    pub fn from_serialized(serialized: String) -> Result<Self> {
        let datalake_type: DynSolType = "(uint256,uint256,uint256,uint256,bytes)".parse()?;
        let bytes = Vec::from_hex(serialized).expect("Invalid hex string");
        let decoded = datalake_type.abi_decode_sequence(&bytes)?;

        let value = decoded.as_tuple().unwrap();
        let datalake_code = value[0].as_uint().unwrap().0.to_string().parse::<usize>()?;

        if datalake_code != 0 {
            bail!("Serialized datalake is not a block datalake");
        }

        let block_range_start = value[1].as_uint().unwrap().0.to_string().parse::<usize>()?;
        let block_range_end = value[2].as_uint().unwrap().0.to_string().parse::<usize>()?;
        let sampled_property = deserialize_sampled_property(value[4].as_bytes().unwrap())?;
        let increment = value[3].as_uint().unwrap().0.to_string().parse::<usize>()?;

        Ok(Self {
            block_range_start,
            block_range_end,
            sampled_property,
            increment,
        })
    }

    pub async fn compile(&self, fetcher: Arc<RwLock<AbstractFetcher>>) -> Result<Vec<String>> {
        compile_block_sampled_datalake(
            self.block_range_start,
            self.block_range_end,
            &self.sampled_property,
            self.increment,
            fetcher,
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
        DatalakeBase::new(&self.to_string(), Datalake::BlockSampled(self.clone()))
    }
}

fn serialize_sampled_property(sampled_property: &str) -> Vec<u8> {
    let tokens: Vec<&str> = sampled_property.split('.').collect();
    let collection = match tokens[0] {
        "header" => Collection::Header,
        "account" => Collection::Account,
        "storage" => Collection::Storage,
        _ => panic!("Unknown collection type"),
    };

    let mut serialized = Vec::new();
    serialized.push(match collection {
        Collection::Header => 1,
        Collection::Account => 2,
        Collection::Storage => 3,
    });

    match collection {
        Collection::Header => {
            let index = HeaderField::from_str(tokens[1].to_uppercase().as_str())
                .unwrap()
                .to_index();
            serialized.push(index as u8);
        }
        Collection::Account | Collection::Storage => {
            // if !is_address(tokens[1]) {
            //     panic!("Invalid account address");
            // }
            let account_bytes = hex::decode(&tokens[1][2..]).expect("Decoding failed");
            serialized.extend_from_slice(&account_bytes);

            if collection == Collection::Account {
                if let Some(index) = AccountField::from_str(tokens[2].to_uppercase().as_str())
                    .unwrap()
                    .to_index()
                {
                    serialized.push(index as u8);
                } else {
                    panic!("Invalid account field");
                }
            } else {
                if tokens[2].len() != 66 || !tokens[2][2..].chars().all(|c| c.is_ascii_hexdigit()) {
                    panic!("Invalid storage slot");
                }
                let slot_bytes = hex::decode(&tokens[2][2..]).expect("Decoding failed");
                serialized.extend_from_slice(&slot_bytes);
            }
        }
    }

    serialized
}

fn deserialize_sampled_property(serialized: &[u8]) -> Result<String> {
    let collection_id = serialized[0] as usize;
    let collection = ["header", "account", "storage"][collection_id - 1];

    match collection {
        "header" => {
            let header_prop_index = serialized[1] as usize;
            let prop = HeaderField::from_index(header_prop_index)
                .ok_or("Invalid header property index")
                .unwrap()
                .as_str();
            Ok(format!("{}.{}", collection, prop.to_lowercase()))
        }
        "account" => {
            let account = Address::from_slice(&serialized[1..21]);
            let account_checksum = format!("{:?}", account);
            let account_prop_index = serialized[21] as usize;
            let prop = AccountField::from_index(account_prop_index)
                .ok_or("Invalid account property index")
                .unwrap()
                .as_str();
            Ok(format!(
                "{}.{}.{}",
                collection,
                account_checksum,
                prop.to_lowercase()
            ))
        }
        "storage" => {
            let account = Address::from_slice(&serialized[1..21]);
            let account_checksum = format!("{:?}", account);
            let slot = &serialized[21..53];
            let slot_string = bytes_to_hex_string(slot);

            Ok(format!(
                "{}.{}.{}",
                collection, account_checksum, slot_string
            ))
        }
        _ => bail!("Invalid collection id"),
    }
}
