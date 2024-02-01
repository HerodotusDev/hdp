use std::{collections::HashMap, str::FromStr};

use anyhow::Result;
use common::block::header::{decode_header_field, HeaderField};
use fetcher::{example_data::get_example_headers, memoizer::Memoizer};

use crate::datalake::base::DataPoint;

// TODO : WIP
pub fn compile_block_sampled_datalake(
    block_range_start: usize,
    block_range_end: usize,
    sampled_property: &str,
    increment: usize,
) -> Result<Vec<DataPoint>> {
    let header_example = get_example_headers();
    let memoizer = Memoizer::pre_filled_memoizer(header_example, HashMap::new(), HashMap::new());
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
            // let property = property_parts[2];
            // Convert property to AccountField enum variant here
            for i in block_range_start..=block_range_end {
                if i % increment != 0 {
                    continue;
                }
                // let acc = memoizer
                //     .get_account(i, account)?
                //     .ok_or(format!("No memoized account for block number: {}", i))?;
                // let value = decode_account_field(&acc, &AccountField::YourFieldHere)?
                //     .ok_or("Decode failed")?;
                aggregation_set.push(DataPoint::Str(account.to_string()));
            }
        }
        "storage" => {
            // let account = property_parts[1];
            let slot = property_parts[2];
            for i in block_range_start..=block_range_end {
                if i % increment != 0 {
                    continue;
                }
                // let value = memoizer
                //     .get_storage_slot(i, account, slot)?
                //     .ok_or(format!("No memoized storage slot for block number: {}", i))?;
                aggregation_set.push(DataPoint::Str(slot.to_string()));
            }
        }
        _ => todo!(),
    }

    Ok(aggregation_set)
}
