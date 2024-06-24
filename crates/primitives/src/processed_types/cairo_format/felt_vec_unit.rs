use alloy_primitives::hex;
use anyhow::Result;
use serde::Serialize;
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;

#[serde_as]
#[derive(Serialize, Debug)]
pub struct FieldElementVectorUnit {
    #[serde_as(as = "Vec<UfeHex>")]
    pub felts: Vec<FieldElement>,
    pub bytes_len: u64,
}

impl FieldElementVectorUnit {
    pub fn from_hex_str(hex_str: &str) -> Result<Self> {
        if hex_str.is_empty() {
            return Err(anyhow::anyhow!("Empty hex input"));
        }
        let bytes = hex::decode(hex_str)?;
        let bytes_len = bytes.len() as u64;
        let felts = bytes
            .chunks(8)
            .map(|chunk| {
                let mut arr = [0u8; 8];
                let len = chunk.len();
                arr[..len].copy_from_slice(chunk);
                let le_int = u64::from_le_bytes(arr);
                FieldElement::from_dec_str(&le_int.to_string())
                    .expect("Invalid to convert FieldElement")
            })
            .collect();

        Ok(Self { felts, bytes_len })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_hex_str() {
        let hex_str = "";
        let result = FieldElementVectorUnit::from_hex_str(hex_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_single_byte_hex_str() {
        let hex_str = "0x01";
        let result = FieldElementVectorUnit::from_hex_str(hex_str).unwrap();
        assert_eq!(result.bytes_len, 1);
        assert_eq!(result.felts.len(), 1);
        assert_eq!(result.felts[0], FieldElement::from_hex_be("0x1").unwrap());
    }

    #[test]
    fn test_non_aligned_hex_str() {
        let hex_str = "0x1234567890abc";
        let result = FieldElementVectorUnit::from_hex_str(hex_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_single_chunk_hex_str() {
        let hex_str = "0x1234567890abcdef";
        let result = FieldElementVectorUnit::from_hex_str(hex_str).unwrap();
        assert_eq!(result.bytes_len, 8);
        assert_eq!(result.felts.len(), 1);
        assert_eq!(
            result.felts[0],
            FieldElement::from_hex_be("efcdab9078563412").unwrap()
        );
    }

    #[test]
    fn test_multiple_chunks_hex_str() {
        let hex_str = "0x1234567890abcdef1122334455667788";
        let result = FieldElementVectorUnit::from_hex_str(hex_str).unwrap();
        assert_eq!(result.bytes_len, 16);
        assert_eq!(result.felts.len(), 2);
        assert_eq!(
            result.felts[0],
            FieldElement::from_hex_be("efcdab9078563412").unwrap()
        );
        assert_eq!(
            result.felts[1],
            FieldElement::from_hex_be("8877665544332211").unwrap()
        );
    }
}
