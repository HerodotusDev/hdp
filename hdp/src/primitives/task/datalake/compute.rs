use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

use crate::primitives::aggregate_fn::{AggregationFunction, FunctionContext};

/// [`Computation`] is a structure that contains the aggregate function id and context
#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
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

impl<'de> Deserialize<'de> for Computation {
    fn deserialize<D>(deserializer: D) -> Result<Computation, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "camelCase")]
        enum Field {
            AggregateFnId,
            AggregateFnCtx,
        }

        struct ComputationVisitor;

        impl<'de> Visitor<'de> for ComputationVisitor {
            type Value = Computation;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Computation")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Computation, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut aggregate_fn_id = None;
                let mut aggregate_fn_ctx = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::AggregateFnId => {
                            if aggregate_fn_id.is_some() {
                                return Err(de::Error::duplicate_field("aggregateFnId"));
                            }
                            aggregate_fn_id = Some(map.next_value()?);
                        }
                        Field::AggregateFnCtx => {
                            if aggregate_fn_ctx.is_some() {
                                return Err(de::Error::duplicate_field("aggregateFnCtx"));
                            }
                            aggregate_fn_ctx = Some(map.next_value()?);
                        }
                    }
                }
                let aggregate_fn_id =
                    aggregate_fn_id.ok_or_else(|| de::Error::missing_field("aggregateFnId"))?;
                let aggregate_fn_ctx = aggregate_fn_ctx.unwrap_or_default();
                Ok(Computation {
                    aggregate_fn_id,
                    aggregate_fn_ctx,
                })
            }
        }

        const FIELDS: &[&str] = &["aggregateFnId", "aggregateFnCtx"];
        deserializer.deserialize_struct("Computation", FIELDS, ComputationVisitor)
    }
}
