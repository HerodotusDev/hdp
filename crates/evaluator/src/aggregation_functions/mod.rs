use types::datalake::base::DataPoint;

pub mod avg;

/// Get [`AggregationFunction`] from function id
pub fn get_aggregation_function_type(function_id: String) -> AggregationFunction {
    match function_id.as_str() {
        "AVG" => AggregationFunction::Avg,
        _ => panic!("Unknown aggregation function"),
    }
}

/// Aggregation function types
pub enum AggregationFunction {
    Avg,
}

impl AggregationFunction {
    pub fn get_index(&self) -> usize {
        match self {
            AggregationFunction::Avg => 0,
        }
    }

    pub fn operation(&self, values: &Vec<DataPoint>) -> usize {
        match self {
            AggregationFunction::Avg => avg::avg(values),
        }
    }
}
