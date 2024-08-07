use hdp::primitives::task::{datalake::DatalakeCompute, module::Module};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Task {
    DatalakeCompute(DatalakeCompute),
    Module(Module),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitBatchQuery {
    pub destination_chain_id: u64,
    pub tasks: Vec<Task>,
}

#[test]
fn test_serialize_submit_batch_query_datalake() {
    let json_data = r#"
    {
      "destinationChainId": 11155111,
      "tasks": [
        {
          "type": "DatalakeCompute",
          "datalake": {
            "type": "BlockSampled",
            "chainId": 11155111,
            "blockRangeStart": 5515020,
            "blockRangeEnd": 5515039,
            "increment": 10,
            "sampledProperty": "header.base_fee_per_gas"
          },
          "compute": {
            "aggregateFnId": "avg"
          }
        },
        {
          "type": "DatalakeCompute",
          "datalake": {
            "type": "TransactionsInBlock",
    
            "chainId": 11155111,
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
          "compute": {
            "aggregateFnId": "count",
            "aggregateFnCtx": {
              "operator": "gt",
              "valueToCompare": "1000000000000000000"
            }
          }
        }
      ]
    }  
    "#;

    let parsed: SubmitBatchQuery = serde_json::from_str(json_data).unwrap();
    println!("{:?}", parsed);
}

#[test]
fn test_serialize_submit_batch_query_module() {
    let json_data = r#"
    {
      "destinationChainId": 11155111,
      "tasks": [
        {
            "type": "Module",
            "programHash": "0xaf1333b8346c1ac941efe380f3122a71c1f7cbad19301543712e74f765bfca",
            "inputs": ["0x4F21E5", "0x4F21E8", "0x13cb6ae34a13a0977f4d7101ebc24b87bb23f0d5"]
        }        
      ]
    }
    "#;

    let parsed: SubmitBatchQuery = serde_json::from_str(json_data).unwrap();
    println!("{:?}", parsed);
}
