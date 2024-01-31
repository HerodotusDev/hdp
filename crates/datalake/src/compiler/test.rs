use anyhow::Result;

use crate::datalake_base::DataPoint;

pub fn test_closer() -> Result<Vec<DataPoint>> {
    println!("test_closer");
    Ok(vec![DataPoint::Int(1)])
}
