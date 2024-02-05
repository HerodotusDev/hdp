use anyhow::Result;
use common::datalake::base::DataPoint;

// TODO: Implement merkleize
pub fn merkleize(_values: &[DataPoint]) -> Result<DataPoint> {
    Ok(DataPoint::Str("".to_string()))
}
