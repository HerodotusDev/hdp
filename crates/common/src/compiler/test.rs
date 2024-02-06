use anyhow::Result;

use crate::datalake::base::DataPoint;

pub async fn test_closer() -> Result<Vec<DataPoint>> {
    println!("test_closer");
    Ok(vec![DataPoint::Int(1)])
}
