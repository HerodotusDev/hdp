use hdp_primitives::task::{datalake::DatalakeCompute, module::Module};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Task {
    DatalakeCompute(DatalakeCompute),
    Module(Module),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitBatchQuery {
    pub delivery_chain_id: u64,
    pub tasks: Vec<Task>,
}

#[test]
fn test_serialize_submit_batch_query() {
    let json_data = r#"
    {
      "deliveryChainId": 11155111,
      "tasks": [
        {
          "datalakeCompute": {
            "datalake": {
              "type": "BlockSampled",
              "datalake": {
                "chainId": 11155111,
                "blockRangeStart": 5515020,
                "blockRangeEnd": 5515039,
                "increment": 10,
                "sampledProperty": "header.base_fee_per_gas"
              }
            },
            "compute": {
              "aggregateFnId": "count",
              "aggregateFnCtx" : {
                "operator":"GreaterThan",
                "valueToCompare": "1000000000000000000"
              }
            }
          }
        }
      ]
    }
    
    "#;

    let parsed: SubmitBatchQuery = serde_json::from_str(json_data).unwrap();
    println!("{:?}", parsed);
}
