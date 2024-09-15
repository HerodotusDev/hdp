use serde::Serialize;
use serde_with::serde_as;
use starknet::core::serde::unsigned_field_element::UfeHex;
use starknet_crypto::FieldElement;

#[serde_as]
#[derive(Serialize, Debug)]
pub struct FieldElementVectorUnit {
    /// Chunked vector of field elements
    #[serde_as(as = "Vec<UfeHex>")]
    pub felts: Vec<FieldElement>,
    /// Length of the original byte array before chunking into field elements
    pub bytes_len: u64,
}

impl FieldElementVectorUnit {
    /// Converts a byte slice into a `FieldElementVectorUnit`.
    ///
    /// This function takes a slice of bytes and converts it into a `FieldElementVectorUnit`,
    /// which consists of a vector of `FieldElement`s and the length of the original byte slice.
    ///
    /// # Panics
    ///
    /// This function will panic if the input byte slice is empty.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        if bytes.is_empty() {
            panic!("Cannot convert to FieldElementVectorUnit from empty bytes")
        }
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

        Self { felts, bytes_len }
    }
}

#[cfg(test)]
mod tests {
    use alloy::hex;

    use super::*;

    #[test]
    #[should_panic(expected = "Cannot convert to FieldElementVectorUnit from empty bytes")]
    fn test_empty_bytes() {
        let bytes = hex::decode("").unwrap();
        FieldElementVectorUnit::from_bytes(&bytes);
    }

    #[test]
    fn test_single_byte_bytes() {
        let bytes = hex::decode("0x01").unwrap();
        let result = FieldElementVectorUnit::from_bytes(&bytes);
        assert_eq!(result.bytes_len, 1);
        assert_eq!(result.felts.len(), 1);
        assert_eq!(result.felts[0], FieldElement::from_hex_be("0x1").unwrap());
    }

    #[test]
    fn test_single_chunk_bytes() {
        let bytes = hex::decode("0x1234567890abcdef").unwrap();
        let result = FieldElementVectorUnit::from_bytes(&bytes);
        assert_eq!(result.bytes_len, 8);
        assert_eq!(result.felts.len(), 1);
        assert_eq!(
            result.felts[0],
            FieldElement::from_hex_be("efcdab9078563412").unwrap()
        );
    }

    #[test]
    fn test_multiple_chunks_bytes() {
        let bytes = hex::decode("0x1234567890abcdef1122334455667788").unwrap();
        let result = FieldElementVectorUnit::from_bytes(&bytes);
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
