use alloy::primitives::ChainId;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct DataProcessorRequest {
    #[serde(rename = "deliveryChainId")]
    pub to_chain_id: ChainId,
    #[serde(rename = "sourceChainId")]
    pub from_chain_id: ChainId,
    pub tasks: Vec<RequestTask>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "taskType", content = "taskContent")]
pub enum RequestTask {
    #[serde(rename = "datalake_compute")]
    DatalakeCompute(RequestDatalakeCompute),
    #[serde(rename = "module_compute")]
    ModuleCompute(RequestModule),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestDatalakeCompute {
    #[serde(rename = "datalakeType")]
    pub datalake_type: String,
    pub datalake: RequestDatalake,
    #[serde(rename = "aggregateFnId")]
    pub aggregate_fn_id: String,
    #[serde(rename = "aggregateFnCtx")]
    pub aggregate_fn_ctx: RequestAggregateFnCtx,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestDatalake {
    #[serde(rename = "targetBlock")]
    pub target_block: u64,
    #[serde(rename = "startIndex")]
    pub start_index: u64,
    #[serde(rename = "endIndex")]
    pub end_index: u64,
    pub increment: u64,
    #[serde(rename = "includedTypes")]
    pub included_types: IncludedTypes,
    #[serde(rename = "sampledProperty")]
    pub sampled_property: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IncludedTypes {
    pub legacy: bool,
    pub eip2930: bool,
    pub eip1559: bool,
    pub eip4844: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestAggregateFnCtx {
    #[serde(rename = "operatorId")]
    pub operator: String,
    #[serde(rename = "valueToCompare")]
    pub value_to_compare: u64,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct RequestModule {
    #[serde_as(as = "UfeHex")]
    #[serde(rename = "class_hash")]
    pub class_hash: FieldElement,
    #[serde_as(as = "Vec<UfeHex>")]
    pub inputs: Vec<FieldElement>,
}

#[cfg(test)]
mod tests {
    use starknet::macros::felt;

    use super::*;

    #[test]
    fn test_deserialize_request() {
        let json_str = r#"
        {
            "deliveryChainId": 11155111,
            "sourceChainId": 11155111,
            "tasks": [
                {
                    "taskType": "datalake_compute",
                    "taskContent": {
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
                        "aggregateFnId": "slr",
                        "aggregateFnCtx": {
                            "operatorId": "none",
                            "valueToCompare": 100
                        }
                    }
                },
                {
                    "taskType": "module_compute",
                    "taskContent": {
                        "class_hash": "0x054af96825d987ca89cf320f7c5a8031017815d884cff1592e8ff6da309f3ca6",
                        "inputs": [
                            "0x1", "0x2", "0x3"
                        ]
                    }
                }
            ]
        }
        "#;

        let request: DataProcessorRequest = serde_json::from_str(json_str).unwrap();
        assert_eq!(request.to_chain_id, 11155111);
        assert_eq!(request.from_chain_id, 11155111);
        assert_eq!(request.tasks.len(), 2);

        let task = &request.tasks[0];
        match task {
            RequestTask::DatalakeCompute(datalake_compute) => {
                assert_eq!(datalake_compute.datalake_type, "transactions_in_block");
                assert_eq!(datalake_compute.aggregate_fn_id, "slr");
                assert_eq!(datalake_compute.aggregate_fn_ctx.operator, "none");
                assert_eq!(datalake_compute.aggregate_fn_ctx.value_to_compare, 100);
            }
            _ => panic!("Expected DatalakeCompute task"),
        }

        let task = &request.tasks[1];
        match task {
            RequestTask::ModuleCompute(module_compute) => {
                assert_eq!(
                    module_compute.class_hash,
                    felt!("0x054af96825d987ca89cf320f7c5a8031017815d884cff1592e8ff6da309f3ca6")
                );
                assert_eq!(module_compute.inputs.len(), 3);
            }
            _ => panic!("Expected ModuleCompute task"),
        }
    }
}
