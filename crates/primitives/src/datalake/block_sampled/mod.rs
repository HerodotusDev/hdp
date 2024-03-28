pub mod collection;
pub mod datalake;
pub mod fields;
pub mod types;

// Export all types
pub use collection::*;
pub use datalake::*;
pub use fields::*;

#[cfg(test)]
mod tests {
    use crate::datalake::{Datalake, DatalakeCollection};

    use super::*;
    use alloy_primitives::{Address, U256};
    use std::str::FromStr;

    #[test]
    fn test_block_datalake_for_header() {
        let encoded_block_sample_datalake = "0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009eb0f600000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f000000000000000000000000000000000000000000000000000000000000";
        let decoded_datalake = BlockSampledDatalake::decode(encoded_block_sample_datalake).unwrap();
        let block_datalake =
            BlockSampledDatalake::new(10399990, 10400000, "header.base_fee_per_gas".to_string(), 1)
                .unwrap();
        assert_eq!(
            decoded_datalake.encode().unwrap(),
            block_datalake.encode().unwrap()
        );

        assert_eq!(
            decoded_datalake.encode().unwrap(),
            encoded_block_sample_datalake
        );

        assert_eq!(
            block_datalake.commit(),
            "0x26365cf5692cc38bca06023b8b62ceb0f6bd959a57e3c453be213d1b71d73732".to_string()
        );

        assert_eq!(
            block_datalake.sampled_property,
            BlockSampledCollection::Header(HeaderField::BaseFeePerGas)
        );
    }

    #[test]
    fn test_block_datalake_for_header_massive() {
        let encoded_block_sample_datalake = "0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009d2a6000000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f000000000000000000000000000000000000000000000000000000000000";
        let decoded_datalake: BlockSampledDatalake =
            BlockSampledDatalake::decode(encoded_block_sample_datalake).unwrap();
        let block_datalake =
            BlockSampledDatalake::new(10300000, 10400000, "header.base_fee_per_gas".to_string(), 1)
                .unwrap();

        assert_eq!(
            decoded_datalake.encode().unwrap(),
            block_datalake.encode().unwrap()
        );

        assert_eq!(
            decoded_datalake.encode().unwrap(),
            encoded_block_sample_datalake
        );

        assert_eq!(
            block_datalake.commit(),
            "0xc21f3b3a49c5bed8b7624d0efc050a2a481f06f627d04212bf1d745d0aa5c6f1".to_string()
        );

        assert_eq!(
            block_datalake.sampled_property,
            BlockSampledCollection::Header(HeaderField::BaseFeePerGas)
        );
    }

    #[test]
    fn test_block_datalake_for_account() {
        let encoded_block_sample_datalake = "0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009eb0f600000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000016027b2f05ce9ae365c3dbf30657e2dc6449989e83d60000000000000000000000";
        let decoded_datalake = BlockSampledDatalake::decode(encoded_block_sample_datalake).unwrap();
        let block_datalake = BlockSampledDatalake::new(
            10399990,
            10400000,
            "account.0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6.nonce".to_string(),
            1,
        )
        .unwrap();
        assert_eq!(decoded_datalake, block_datalake);
        assert_eq!(
            decoded_datalake.encode().unwrap(),
            encoded_block_sample_datalake
        );

        assert_eq!(
            block_datalake.commit(),
            "0x79b0d86f9b08c78f527666d4d39d01349530ced0a3d37f4c63e7108814a670b7".to_string()
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
    fn test_block_datalake_for_account_2() {
        let encoded_block_sample_datalake = "0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004b902400000000000000000000000000000000000000000000000000000000004b9027000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000016020a4de450feb156a2a51ed159b2fb99da26e5f3a30000000000000000000000";
        let decoded_datalake = BlockSampledDatalake::decode(encoded_block_sample_datalake).unwrap();
        let block_datalake = BlockSampledDatalake::new(
            4952100,
            4952103,
            "account.0x0a4de450feb156a2a51ed159b2fb99da26e5f3a3.nonce".to_string(),
            1,
        )
        .unwrap();
        let serialized = block_datalake.encode().unwrap();
        assert_eq!(serialized, encoded_block_sample_datalake);
        assert_eq!(decoded_datalake, block_datalake);
        assert_eq!(
            decoded_datalake.encode().unwrap(),
            encoded_block_sample_datalake
        );

        assert_eq!(
            block_datalake.commit(),
            "0x6db54c04174bd625449785ca58efd313e016b807d0a17add522d74e0e27c3b08".to_string()
        );

        assert_eq!(
            block_datalake.sampled_property,
            BlockSampledCollection::Account(
                Address::from_str("0x0a4de450feb156a2a51ed159b2fb99da26e5f3a3").unwrap(),
                AccountField::Nonce
            )
        );
    }

    #[test]
    fn test_block_datalake_for_storage() {
        let encoded_block_sample_datalake = "0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009eb0f600000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000035037b2f05ce9ae365c3dbf30657e2dc6449989e83d600000000000000000000000000000000000000000000000000000000000000ff0000000000000000000000";
        let decoded_datalake = BlockSampledDatalake::decode(encoded_block_sample_datalake).unwrap();
        let block_datalake = BlockSampledDatalake::new(
            10399990,
            10400000,
            "storage.0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6.0x00000000000000000000000000000000000000000000000000000000000000ff".to_string(),
            1,
        ).unwrap();
        assert_eq!(decoded_datalake, block_datalake);
        assert_eq!(
            decoded_datalake.encode().unwrap(),
            encoded_block_sample_datalake
        );

        assert_eq!(
            block_datalake.commit(),
            "0x147dc75fd577a75dca31c0c5181539a1078c48759e379685b827f8c0e3f0b6ef".to_string()
        );

        assert_eq!(
            block_datalake.sampled_property,
            BlockSampledCollection::Storage(
                Address::from_str("0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6").unwrap(),
                U256::from(0xff)
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
            U256::from(0xfffe),
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
