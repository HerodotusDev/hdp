use types::datalake::base::DataPoint;

// TODO: Implement the avg function
pub fn avg(values: &Vec<DataPoint>) -> usize {
    let mut sum = 0;
    for value in values {
        match value {
            DataPoint::Str(_) => panic!("String value found"),
            DataPoint::Int(int) => sum += int,
        }
    }
    sum / values.len()
}
