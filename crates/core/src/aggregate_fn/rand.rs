use alloy_primitives::U256;
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use super::{integer::Operator, AggregationFunction, FunctionContext};

impl Distribution<AggregationFunction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> AggregationFunction {
        let index: u8 = rng.gen_range(0..=5);
        AggregationFunction::from_index(index).unwrap()
    }
}

impl Distribution<Operator> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Operator {
        let index: u8 = rng.gen_range(1..=6);
        Operator::from_index(index).unwrap().unwrap()
    }
}

impl Distribution<FunctionContext> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> FunctionContext {
        FunctionContext {
            operator: rng.sample(Standard),
            value_to_compare: U256::from(rng.gen_range(0..=100)),
        }
    }
}
