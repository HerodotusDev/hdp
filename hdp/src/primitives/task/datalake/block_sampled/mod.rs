pub mod collection;
pub mod datalake;
pub mod rlp_fields;

// Export all types
pub use collection::*;
pub use datalake::*;
pub use rlp_fields::*;

#[cfg(test)]
mod tests {
    use crate::{
        primitives::solidity_types::traits::DatalakeCodecs,
        primitives::task::datalake::DatalakeCollection,
    };

    use super::*;
    use alloy::{
        hex,
        primitives::{Address, StorageKey},
    };
    use std::str::FromStr;

    #[test]
    fn test_block_datalake_for_header() {
        let encoded_block_sample_datalake = hex::decode("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa36a700000000000000000000000000000000000000000000000000000000009eb0f600000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000002010f000000000000000000000000000000000000000000000000000000000000").unwrap();
        let decoded_datalake =
            BlockSampledDatalake::decode(&encoded_block_sample_datalake).unwrap();
        let block_datalake = BlockSampledDatalake::new(
            11155111,
            10399990,
            10400000,
            1,
            "header.base_fee_per_gas".parse().unwrap(),
        );

        assert_eq!(
            decoded_datalake.encode().unwrap(),
            block_datalake.encode().unwrap()
        );

        assert_eq!(
            decoded_datalake.encode().unwrap(),
            encoded_block_sample_datalake
        );

        assert_eq!(
            block_datalake.commit().to_string(),
            "0xdd01208038666c84be0142986d50b65100af3ba5194b6be2838659f5bf40614b".to_string()
        );

        assert_eq!(
            block_datalake.sampled_property,
            BlockSampledCollection::Header(HeaderField::BaseFeePerGas)
        );
    }

    #[test]
    fn test_block_datalake_for_account() {
        let encoded_block_sample_datalake = hex::decode("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa36a700000000000000000000000000000000000000000000000000000000009eb0f600000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000016027b2f05ce9ae365c3dbf30657e2dc6449989e83d60000000000000000000000").unwrap();
        let decoded_datalake =
            BlockSampledDatalake::decode(&encoded_block_sample_datalake).unwrap();
        let block_datalake = BlockSampledDatalake::new(
            11155111,
            10399990,
            10400000,
            1,
            "account.0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6.nonce"
                .parse()
                .unwrap(),
        );

        assert_eq!(decoded_datalake, block_datalake);
        assert_eq!(
            decoded_datalake.encode().unwrap(),
            encoded_block_sample_datalake
        );

        assert_eq!(
            block_datalake.commit().to_string(),
            "0x02586066a48c2781a9a18b1322ea5913deb96cc1eafaea26b788eeba647d6994".to_string()
        );

        assert_eq!(
            block_datalake.sampled_property,
            BlockSampledCollection::Account(
                Address::from_str("0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6").unwrap(),
                AccountField::Nonce
            )
        );
    }

    #[test]
    fn test_block_datalake_for_storage() {
        let encoded_block_sample_datalake = hex::decode("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000aa36a700000000000000000000000000000000000000000000000000000000009eb0f600000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000000035037b2f05ce9ae365c3dbf30657e2dc6449989e83d600000000000000000000000000000000000000000000000000000000000000ff0000000000000000000000").unwrap();
        let decoded_datalake =
            BlockSampledDatalake::decode(&encoded_block_sample_datalake).unwrap();
        let block_datalake = BlockSampledDatalake::new(
            11155111,
            10399990,
            10400000,
            1,
            "storage.0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6.0x00000000000000000000000000000000000000000000000000000000000000ff".parse().unwrap(),
        );

        assert_eq!(decoded_datalake, block_datalake);
        assert_eq!(
            decoded_datalake.encode().unwrap(),
            encoded_block_sample_datalake
        );

        assert_eq!(
            block_datalake.commit().to_string(),
            "0x2df3f9a445c0ec26efd38b5f23828498f1c41d940804baded3923dada6ae1415".to_string()
        );

        assert_eq!(
            block_datalake.sampled_property,
            BlockSampledCollection::Storage(
                Address::from_str("0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6").unwrap(),
                StorageKey::from_str(
                    "0x00000000000000000000000000000000000000000000000000000000000000ff"
                )
                .unwrap()
            )
        );
    }

    #[test]
    fn test_header_collection_serialize() {
        let header_collection = BlockSampledCollection::Header(HeaderField::BaseFeePerGas);
        let serialized = header_collection.serialize().unwrap();
        assert_eq!(serialized, [1, 15]);

        let header_collection = BlockSampledCollection::Header(HeaderField::Difficulty);
        let serialized = header_collection.serialize().unwrap();
        assert_eq!(serialized, [1, 7]);
    }

    #[test]
    fn test_account_collection_serialize() {
        let account_collection = BlockSampledCollection::Account(
            Address::from_str("0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6").unwrap(),
            AccountField::Nonce,
        );

        let serialized = account_collection.serialize().unwrap();
        assert_eq!(
            serialized,
            [
                2, 123, 47, 5, 206, 154, 227, 101, 195, 219, 243, 6, 87, 226, 220, 100, 73, 152,
                158, 131, 214, 0
            ]
        );

        let account_collection = BlockSampledCollection::Account(
            Address::from_str("0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6").unwrap(),
            AccountField::Balance,
        );

        let serialized = account_collection.serialize().unwrap();
        assert_eq!(
            serialized,
            [
                2, 123, 47, 5, 206, 154, 227, 101, 195, 219, 243, 6, 87, 226, 220, 100, 73, 152,
                158, 131, 214, 1
            ]
        );
    }

    #[test]
    fn test_storage_collection_serialize() {
        let storage_collection = BlockSampledCollection::Storage(
            Address::from_str("0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6").unwrap(),
            StorageKey::from_str(
                "0x000000000000000000000000000000000000000000000000000000000000fffe",
            )
            .unwrap(),
        );

        let serialized = storage_collection.serialize().unwrap();
        assert_eq!(
            serialized,
            [
                3, 123, 47, 5, 206, 154, 227, 101, 195, 219, 243, 6, 87, 226, 220, 100, 73, 152,
                158, 131, 214, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 255, 254
            ]
        )
    }
}
