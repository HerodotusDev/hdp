use std::str::FromStr;

use crate::aggregate_fn::{integer::Operator, AggregationFunction, FunctionContext};
use alloy::primitives::U256;

/// [`Computation`] is a structure that contains the aggregate function id and context
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Computation {
    pub aggregate_fn_id: AggregationFunction,
    pub aggregate_fn_ctx: FunctionContext,
}

impl Computation {
    pub fn new(aggregate_fn_id: &str, aggregate_fn_ctx: Option<FunctionContext>) -> Self {
        let aggregate_fn_ctn_parsed = match aggregate_fn_ctx {
            None => FunctionContext::new(Operator::None, U256::ZERO),
            Some(ctx) => ctx,
        };
        Self {
            aggregate_fn_id: AggregationFunction::from_str(aggregate_fn_id).unwrap(),
            aggregate_fn_ctx: aggregate_fn_ctn_parsed,
        }
    }
}
