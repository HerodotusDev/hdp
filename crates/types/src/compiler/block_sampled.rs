use std::{collections::HashMap, str::FromStr};

use anyhow::Result;
use common::block::{
    account::{decode_account_field, AccountField},
    header::{decode_header_field, HeaderField},
};
use fetcher::{
    example_data::{get_example_accounts, get_example_headers},
    memoizer::Memoizer,
};

use crate::datalake::base::DataPoint;

// TODO : WIP
pub fn compile_block_sampled_datalake(
    block_range_start: usize,
    block_range_end: usize,
    sampled_property: &str,
    increment: usize,
) -> Result<Vec<DataPoint>> {
    let prefilled_header = get_example_headers();
    let prefilled_account = get_example_accounts();
    let memoizer =
        Memoizer::pre_filled_memoizer(prefilled_header, prefilled_account, HashMap::new());
    let property_parts: Vec<&str> = sampled_property.split('.').collect();
    let collection = property_parts[0];

    let mut aggregation_set: Vec<DataPoint> = Vec::new();

    match collection {
        "header" => {
            let property = property_parts[1];

            for i in block_range_start..=block_range_end {
                if i % increment != 0 {
                    continue;
                }
                let header = memoizer.get_rlp_header(i).unwrap();
                let value = decode_header_field(
                    &header,
                    HeaderField::from_str(&property.to_uppercase()).unwrap(),
                );

                aggregation_set.push(DataPoint::Str(value));
            }
        }
        "account" => {
            let account = property_parts[1];
            let property = property_parts[2];

            for i in block_range_start..=block_range_end {
                if i % increment != 0 {
                    continue;
                }
                let acc = memoizer.get_rlp_account(i, account.to_string()).unwrap();
                let value = decode_account_field(
                    &acc,
                    AccountField::from_str(&property.to_uppercase()).unwrap(),
                );

                aggregation_set.push(DataPoint::Str(value));
            }
        }
        "storage" => {
            let account = property_parts[1];
            let slot = property_parts[2];
            for i in block_range_start..=block_range_end {
                if i % increment != 0 {
                    continue;
                }
                let value = memoizer
                    .get_rlp_storage(i, account.to_string(), slot.to_string())
                    .unwrap();
                aggregation_set.push(DataPoint::Str(value));
            }
        }
        _ => todo!(),
    }

    Ok(aggregation_set)
}
