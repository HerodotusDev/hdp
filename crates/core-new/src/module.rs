//! Module is the unit of pre-processing.
//! It contains the hash and the input.
//! This is request interface for the preprocessor.

use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use serde::Serialize;
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;

#[serde_as]
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct Module {
    #[serde_as(as = "UfeHex")]
    pub class_hash: FieldElement,
    #[serde_as(as = "Vec<UfeHex>")]
    pub inputs: Vec<FieldElement>,
}

pub enum ModuleTag {
    TEST,
}

impl Module {
    pub fn from_tag(tag: ModuleTag, inputs: Vec<FieldElement>) -> Self {
        let class_hash = match tag {
            ModuleTag::TEST => FieldElement::from_hex_be(
                "0x054af96825d987ca89cf320f7c5a8031017815d884cff1592e8ff6da309f3ca6",
            ),
        }
        .expect("Invalid module tag");
        Self { class_hash, inputs }
    }

    pub fn new(class_hash: FieldElement, inputs: Vec<FieldElement>) -> Self {
        Self { class_hash, inputs }
    }

    pub fn get_class_hash(&self) -> FieldElement {
        self.class_hash
    }

    pub fn get_module_inputs(&self) -> Vec<FieldElement> {
        self.inputs.clone()
    }
}

#[serde_as]
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ModuleWithClass {
    pub module: Module,
    pub module_class: CasmContractClass,
}

impl ModuleWithClass {
    pub fn new(module: Module, module_class: CasmContractClass) -> Self {
        Self {
            module,
            module_class,
        }
    }

    pub fn get_module(&self) -> Module {
        self.module.clone()
    }

    pub fn get_class(&self) -> CasmContractClass {
        self.module_class.clone()
    }
}
