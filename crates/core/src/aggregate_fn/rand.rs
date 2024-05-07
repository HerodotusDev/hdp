use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use super::{integer::Operator, AggregationFunction};

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
