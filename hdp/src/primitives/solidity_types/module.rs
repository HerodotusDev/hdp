use alloy::{
    dyn_abi::DynSolValue,
    hex,
    primitives::{keccak256, B256},
};
use tracing::debug;

use crate::primitives::{task::module::Module, utils::felt_to_bytes32};

impl Module {
    pub fn encode_task(&self) -> Vec<u8> {
        let class_hash: DynSolValue =
            DynSolValue::FixedBytes(felt_to_bytes32(self.program_hash), 32);
        let module_inputs: DynSolValue = DynSolValue::FixedArray(
            self.get_public_inputs()
                .iter()
                .map(|input| DynSolValue::FixedBytes(felt_to_bytes32(*input), 32))
                .collect(),
        );
        let input_length: DynSolValue = self.get_public_inputs().len().into();
        // offset of class hash
        let offset: DynSolValue = (64).into();
        let module_tuple_value =
            DynSolValue::Tuple(vec![class_hash, offset, input_length, module_inputs]);
        module_tuple_value.abi_encode()
    }

    pub fn commit(&self) -> B256 {
        let encoded_task = self.encode_task();
        debug!("encoded_task: {:?}", hex::encode(encoded_task.clone()));
        keccak256(encoded_task)
    }
}

#[cfg(test)]
mod tests {
    use alloy::primitives::b256;
    use starknet_crypto::FieldElement;

    use crate::primitives::task::module::{ModuleInput, Visibility};

    use super::*;

    #[test]
    pub fn module_encode() {
        let module = Module {
            program_hash: FieldElement::from_hex_be(
                "0x00af1333b8346c1ac941efe380f3122a71c1f7cbad19301543712e74f765bfca",
            )
            .unwrap(),
            inputs: vec![
                ModuleInput::new(Visibility::Public, "0x4F21E5"),
                ModuleInput::new(Visibility::Public, "0x4F21E8"),
                ModuleInput::new(
                    Visibility::Public,
                    "0x13cb6ae34a13a0977f4d7101ebc24b87bb23f0d5",
                ),
            ],
            local_class_path: None,
        };

        let expected_commit = module.commit();
        assert_eq!(
            expected_commit,
            b256!("879869b6d237b92bfdd3f3f7b76baaa9ebb2a3ad5e8478d12cca258d3def05af")
        );
    }

    #[test]
    pub fn module_encode_with_private_input() {
        let module = Module {
            program_hash: FieldElement::from_hex_be(
                "0x00af1333b8346c1ac941efe380f3122a71c1f7cbad19301543712e74f765bfca",
            )
            .unwrap(),
            inputs: vec![
                ModuleInput::new(Visibility::Public, "0x4F21E5"),
                ModuleInput::new(Visibility::Public, "0x4F21E8"),
                ModuleInput::new(
                    Visibility::Private,
                    "0x13cb6ae34a13a0977f4d7101ebc24b87bb23f0d5",
                ),
            ],
            local_class_path: None,
        };

        let expected_commit = module.commit();
        assert_eq!(
            expected_commit,
            b256!("d81ebd27b719967e1df4edf64c9e3ce87635089e3462306af340a393625d8726")
        );
    }
}
