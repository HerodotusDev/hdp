use alloy::{
    dyn_abi::DynSolValue,
    primitives::{keccak256, B256},
};
use anyhow::Result;

use crate::{task::module::Module, utils::felt_to_bytes32};

impl Module {
    pub fn encode(&self) -> Result<Vec<u8>> {
        let task_code: DynSolValue = 1.into();
        let class_hash: DynSolValue = DynSolValue::FixedBytes(felt_to_bytes32(self.class_hash), 32);
        let module_inputs: DynSolValue = DynSolValue::FixedArray(
            self.inputs
                .iter()
                .map(|input| DynSolValue::FixedBytes(felt_to_bytes32(*input), 32))
                .collect(),
        );

        let module_tuple_value = DynSolValue::Tuple(vec![task_code, class_hash, module_inputs]);
        Ok(module_tuple_value.abi_encode())
    }

    pub fn commit(&self) -> B256 {
        let class_hash: DynSolValue = DynSolValue::FixedBytes(felt_to_bytes32(self.class_hash), 32);

        let module_inputs: DynSolValue = DynSolValue::FixedArray(
            self.inputs
                .iter()
                .map(|input| DynSolValue::FixedBytes(felt_to_bytes32(*input), 32))
                .collect(),
        );
        let input_length: DynSolValue = self.inputs.len().into();
        // offset of class hash
        let offset: DynSolValue = (64).into();
        let module_tuple_value =
            DynSolValue::Tuple(vec![class_hash, offset, input_length, module_inputs]);
        keccak256(module_tuple_value.abi_encode())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy::hex;
    use starknet_crypto::FieldElement;

    use super::*;

    #[test]
    pub fn module_encode() {
        let module = Module {
            class_hash: FieldElement::from_hex_be(
                "0x034d4ff54bc5c6cfee6719bfaa94ffa374071e8d656b74823681a955e9033dd9",
            )
            .unwrap(),
            inputs: vec![
                FieldElement::from_hex_be("0x4F21E5").unwrap(),
                FieldElement::from_hex_be("0x4F21E8").unwrap(),
                FieldElement::from_hex_be("0x13cb6ae34a13a0977f4d7101ebc24b87bb23f0d5").unwrap(),
            ],
            local_class_path: None,
        };
        let encoded = module.encode().unwrap();
        let expected = hex::encode(encoded);
        println!("{}", expected);

        let expected_commit = module.commit();
        assert_eq!(
            expected_commit,
            B256::from_str("0x96ed7a050dc775d5a3091181336837f6d807286150878e16d66df377df2fe89a")
                .unwrap()
        )
    }
}
