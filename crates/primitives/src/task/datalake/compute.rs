use serde::{Deserialize, Serialize};

use crate::aggregate_fn::{AggregationFunction, FunctionContext};

/// [`Computation`] is a structure that contains the aggregate function id and context
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Computation {
    pub aggregate_fn_id: AggregationFunction,
    pub aggregate_fn_ctx: FunctionContext,
}

impl Computation {
    pub fn new(
        aggregate_fn_id: AggregationFunction,
        aggregate_fn_ctx: Option<FunctionContext>,
    ) -> Self {
        let aggregate_fn_ctn_parsed = aggregate_fn_ctx.unwrap_or_default();
        Self {
            aggregate_fn_id,
            aggregate_fn_ctx: aggregate_fn_ctn_parsed,
        }
    }
}
