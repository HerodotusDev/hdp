use crate::primitives::task::{datalake::DatalakeCompute, module::Module};
use serde::{Deserialize, Serialize};

use super::chain_id::ChainId;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Task {
    DatalakeCompute(DatalakeCompute),
    Module(Module),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubmitBatchQuery {
    pub destination_chain_id: ChainId,
    pub tasks: Vec<Task>,
}

#[test]
fn test_serialize_submit_batch_query_datalake() {
    let json_data = r#"
    {
      "destinationChainId": "ETH_SEPOLIA",
      "tasks": [
        {
          "type": "DatalakeCompute",
          "datalake": {
            "type": "BlockSampled",
            "chainId": "ETH_SEPOLIA",
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
            "chainId": "ETH_SEPOLIA",
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
    "destinationChainId": "ETH_SEPOLIA",
    "tasks": [
        {
        "type": "Module",
        "programHash": "0x64041a339b1edd10de83cf031cfa938645450f971d2527c90d4c2ce68d7d412",
        "inputs": [
            {
            "visibility": "private",
            "value": "0x5222a4"
            },
            {
            "visibility": "public",
            "value": "0x00000000000000000000000013cb6ae34a13a0977f4d7101ebc24b87bb23f0d5"
            }
        ]
        }
    ]
    }

    "#;

    let parsed: SubmitBatchQuery = serde_json::from_str(json_data).unwrap();
    println!("{:?}", parsed);
}
