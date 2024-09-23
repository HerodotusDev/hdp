use anyhow::Result;
use serde::Serialize;
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::Felt;

#[serde_as]
#[derive(Serialize, Debug)]
pub struct FieldElementVectorUnit {
    #[serde_as(as = "Vec<UfeHex>")]
    pub felts: Vec<Felt>,
    pub bytes_len: u64,
}

impl FieldElementVectorUnit {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.is_empty() {
            return Err(anyhow::anyhow!("Empty hex input"));
        }
        let bytes_len = bytes.len() as u64;
        let felts = bytes
            .chunks(8)
            .map(|chunk| {
                let mut arr = [0u8; 8];
                let len = chunk.len();
                arr[..len].copy_from_slice(chunk);
                let le_int = u64::from_le_bytes(arr);
                Felt::from_dec_str(&le_int.to_string()).expect("Invalid to convert FieldElement")
            })
            .collect();

        Ok(Self { felts, bytes_len })
    }
}

#[cfg(test)]
mod tests {
    use alloy::hex;

    use super::*;

    #[test]
    fn test_empty_bytes() {
        let bytes = hex::decode("").unwrap();
        let result = FieldElementVectorUnit::from_bytes(&bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_single_byte_bytes() {
        let bytes = hex::decode("0x01").unwrap();
        let result = FieldElementVectorUnit::from_bytes(&bytes).unwrap();
        assert_eq!(result.bytes_len, 1);
        assert_eq!(result.felts.len(), 1);
        assert_eq!(result.felts[0], Felt::from_hex("0x1").unwrap());
    }

    #[test]
    fn test_single_chunk_bytes() {
        let bytes = hex::decode("0x1234567890abcdef").unwrap();
        let result = FieldElementVectorUnit::from_bytes(&bytes).unwrap();
        assert_eq!(result.bytes_len, 8);
        assert_eq!(result.felts.len(), 1);
        assert_eq!(result.felts[0], Felt::from_hex("efcdab9078563412").unwrap());
    }

    #[test]
    fn test_multiple_chunks_bytes() {
        let bytes = hex::decode("0x1234567890abcdef1122334455667788").unwrap();
        let result = FieldElementVectorUnit::from_bytes(&bytes).unwrap();
        assert_eq!(result.bytes_len, 16);
        assert_eq!(result.felts.len(), 2);
        assert_eq!(result.felts[0], Felt::from_hex("efcdab9078563412").unwrap());
        assert_eq!(result.felts[1], Felt::from_hex("8877665544332211").unwrap());
    }
}
