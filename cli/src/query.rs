use hdp_primitives::aggregate_fn::{AggregationFunction, FunctionContext};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct IncludedTypes {
    pub legacy: bool,
    pub eip2930: bool,
    pub eip1559: bool,
    pub eip4844: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockDatalake {
    pub block_range_start: u64,
    pub block_range_end: u64,
    pub increment: Option<u64>,
    pub sampled_property: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionDatalake {
    pub target_block: u64,
    pub start_index: u64,
    pub end_index: u64,
    pub increment: Option<u64>,
    pub included_types: IncludedTypes,
    pub sampled_property: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Datalake {
    Block(BlockDatalake),
    Transaction(TransactionDatalake),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatalakeType {
    BlockSampled,
    TransactionsInBlock,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub datalake_type: DatalakeType,
    pub datalake: Datalake,
    pub aggregate_fn_id: AggregationFunction,
    pub aggregate_fn_ctx: Option<FunctionContext>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitBatchQuery {
    pub delivery_chain_id: u64,
    pub source_chain_id: u64,
    pub tasks: Vec<Task>,
}

#[test]
fn test_serialize_submit_batch_query() {
    let json_data = r#"
    {
        "deliveryChainId": 11155111,
        "sourceChainId": 11155111,
         "tasks": [
             {
             "datalakeType": "block_sampled",
             "datalake": {
               "blockRangeStart": 5515020,
               "blockRangeEnd": 5515039,
               "sampledProperty": "header.base_fee_per_gas"
             },
             "aggregateFnId": "avg"
           },
           {
             "datalakeType": "transactions_in_block",
             "datalake": {
               "targetBlock": 5409986,
               "startIndex": 10,
               "endIndex": 40,
               "increment": 10,
               "includedTypes": {
                 "legacy": true,
                 "eip2930": true,
                 "eip1559": true,
                 "eip4844": true
               },
               "sampledProperty": "tx_receipt.success"
             },
             "aggregateFnId": "max"
           }
         ]
       }
    "#;

    let parsed: SubmitBatchQuery = serde_json::from_str(json_data).unwrap();
    println!("{:?}", parsed);
}
