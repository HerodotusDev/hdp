use anyhow::{bail, Result};

use crate::datalake::base::DataPoint;

// TODO : WIP
pub fn get_aggregation_set_from_expression(
    aggregatable_field: &str,
    block_range_start: i32,
    block_range_end: i32,
    skip_every_nth_block: i32,
) -> Result<Vec<DataPoint>> {
    let parts: Vec<&str> = aggregatable_field.split('.').collect();
    let collection = parts[0];

    let mut aggregation_set: Vec<DataPoint> = Vec::new();

    match collection {
        "header" => {
            let property = parts.get(1).ok_or("Invalid property for header").unwrap();
            // Convert property to HeaderField enum variant here
            for i in block_range_start..=block_range_end {
                if i % skip_every_nth_block != 0 {
                    continue;
                }
                // let header = memoizer
                //     .get_header(i)?
                //     .ok_or(format!("No memoized header for block number: {}", i))?;
                // let value = decode_header_field(&header, &HeaderField::YourFieldHere)?
                //     .ok_or("Decode failed")?;
                aggregation_set.push(DataPoint::Int(i));
            }
        }
        "account" => {
            let account = parts.get(1).ok_or("Invalid account identifier").unwrap();
            let property = parts.get(2).ok_or("Invalid property for account").unwrap();
            // Convert property to AccountField enum variant here
            for i in block_range_start..=block_range_end {
                if i % skip_every_nth_block != 0 {
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
            let account = parts.get(1).ok_or("Invalid account identifier").unwrap();
            let slot = parts.get(2).ok_or("Invalid slot").unwrap();
            for i in block_range_start..=block_range_end {
                if i % skip_every_nth_block != 0 {
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
