use serde::{Deserialize, Serialize};

use crate::primitives::{block::header::MMRMetaFromNewIndexer, utils::hex_string_to_uint, ChainId};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub struct MMRMeta {
    pub id: u64,
    pub root: String,
    pub size: u64,
    // hex encoded
    pub peaks: Vec<String>,
    pub chain_id: u128,
}

impl MMRMeta {
    pub fn new(id: u64, root: String, size: u64, peaks: Vec<String>, chain_id: u128) -> Self {
        MMRMeta {
            id,
            root,
            size,
            peaks,
            chain_id,
        }
    }
}

impl MMRMeta {
    pub fn from_indexer(val: MMRMetaFromNewIndexer, chain_id: ChainId) -> Self {
        MMRMeta {
            id: hex_string_to_uint(&val.mmr_id),
            root: val.mmr_root,
            size: val.mmr_size,
            peaks: val.mmr_peaks,
            chain_id: chain_id.to_numeric_id(),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_mmr_meta() {
        let mmr_meta = MMRMeta::new(
            26,
            "0x18e672dd525cd5eacc5f6b15e5d89451bce65177881304d5200af38e350ebdc".to_string(),
            12273288,
            vec![
                "0x262c4c9b1cb2a036924aecf563dc9952e5f8b41004310adde86f22abb793eb1".to_string(),
                "0x72f553aac8690d09c95fe6220fdd5a073440631e4ca0a161a92b655d2ac9478".to_string(),
                "0x6c68dfa085af40218620038d05f477fba52c4b12b812b64902663abf78bca62".to_string(),
                "0x52a50beb6cbeffbd5db875d77e4d3917fdee5f723165f139dc04fe20cd4d69a".to_string(),
                "0x5c4814bbd601bffb5ac9980977a79bf100d4c1ad4f2caa410f7a7c4249a2fd4".to_string(),
                "0x668035a3620690024dac08a8db46e3316619e4c2a634daaa3175ab16af72deb".to_string(),
                "0x67cff2a39ca6fb235decefaf5bb63f54c550b97b57e9873751eb9dae35cfcd4".to_string(),
                "0x2a7d9ca4745f200dd2c66d2dfd6374a21f7092452287696c395f62afc22c805".to_string(),
                "0x37511dd8cc41503f6c08879d18f15b9ae649d6b2cdd91bcaa3990aeb87ba8c6".to_string(),
                "0x55112088a2f7dfaf5d88ce949f3aad7c7d05d6e4eaff4053aebfbed3af885af".to_string(),
                "0x66c82fce8bfc291095c6c9255b1f7ccf725a1e91e8ae8cd8c43ceb111c21480".to_string(),
                "0x2e5274895f9cd556bb8dee5b2551e9cda9aa3caa23532f9824abcc62d5ad273".to_string(),
            ],
            11155111,
        );

        let processed_string = include_str!("../../../../fixtures/primitives/mmr.json");
        let mmr: MMRMeta = serde_json::from_str(processed_string).unwrap();
        assert_eq!(mmr_meta, mmr);
    }
}
