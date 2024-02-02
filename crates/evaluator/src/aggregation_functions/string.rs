use anyhow::Result;
use types::datalake::base::DataPoint;

pub fn merkleize(_values: &[DataPoint]) -> Result<DataPoint> {
    Ok(DataPoint::Str("".to_string()))
}
