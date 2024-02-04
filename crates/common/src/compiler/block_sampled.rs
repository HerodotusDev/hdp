use std::str::FromStr;

use crate::block::{
    account::{decode_account_field, AccountField},
    header::{decode_header_field, HeaderField},
};
use anyhow::Result;
use fetcher::{
    example_data::{get_example_accounts, get_example_headers, get_example_storages},
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
    // TODO: This is a temporary solution to get the example data later we will add fetcher & memoizer logic
    let prefilled_header = get_example_headers();
    let prefilled_account = get_example_accounts();
    let prefilled_storage = get_example_storages();
    let memoizer =
        Memoizer::pre_filled_memoizer(prefilled_header, prefilled_account, prefilled_storage);
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
                let acc = memoizer.get_rlp_account(i, account.to_string()).unwrap();
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

                let value = memoizer
                    .get_storage_value(i, account.to_string(), slot.to_string())
                    .unwrap();

                aggregation_set.push(DataPoint::Str(value));
            }
        }
        _ => todo!(),
    }

    Ok(aggregation_set)
}
