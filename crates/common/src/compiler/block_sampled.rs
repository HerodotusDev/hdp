use std::{str::FromStr, sync::Arc};

use crate::{
    block::{
        account::{decode_account_field, AccountField},
        header::{decode_header_field, HeaderField},
    },
    fetcher::AbstractFetcher,
};
use anyhow::Result;
use tokio::sync::RwLock;

pub async fn compile_block_sampled_datalake(
    block_range_start: usize,
    block_range_end: usize,
    sampled_property: &str,
    increment: usize,
    fetcher: Arc<RwLock<AbstractFetcher>>,
) -> Result<Vec<String>> {
    let mut abstract_fetcher = fetcher.write().await;
    let property_parts: Vec<&str> = sampled_property.split('.').collect();
    let collection = property_parts[0];

    let mut aggregation_set: Vec<String> = Vec::new();

    match collection {
        "header" => {
            let property = property_parts[1];

            for i in block_range_start..=block_range_end {
                if i % increment != 0 {
                    continue;
                }
                let header = abstract_fetcher.get_rlp_header(i).await;
                let value = decode_header_field(
                    &header,
                    HeaderField::from_str(&property.to_uppercase()).unwrap(),
                );

                aggregation_set.push(value);
            }
        }
        "account" => {
            let account = property_parts[1];
            let property = property_parts[2];

            for i in block_range_start..=block_range_end {
                if i % increment != 0 {
                    continue;
                }
                let acc = abstract_fetcher
                    .get_rlp_account(i, account.to_string())
                    .await;

                let value = decode_account_field(
                    &acc,
                    AccountField::from_str(&property.to_uppercase()).unwrap(),
                );

                aggregation_set.push(value);
            }
        }
        "storage" => {
            let account = property_parts[1];
            let slot = property_parts[2];

            for i in block_range_start..=block_range_end {
                if i % increment != 0 {
                    continue;
                }

                let value = abstract_fetcher
                    .get_storage_value(i, account.to_string(), slot.to_string())
                    .await;

                aggregation_set.push(value);
            }
        }
        _ => todo!(),
    }

    Ok(aggregation_set)
}
