use types::datalake::{base::DataPoint, block_sampled::BlockSampledDatalake};

#[test]
fn test_block_datalake_for_header() {
    let blocksample_datalake = "0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009eb0f600000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f000000000000000000000000000000000000000000000000000000000000";
    let decoded_datalake =
        BlockSampledDatalake::from_serialized(blocksample_datalake.to_string()).unwrap();
    let block_datalake =
        BlockSampledDatalake::new(10399990, 10400000, "header.base_fee_per_gas".to_string(), 1);
    assert_eq!(decoded_datalake, block_datalake);
}

#[test]
fn test_block_datalake_for_account() {
    let blocksample_datalake = "0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009eb0f600000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000016027b2f05ce9ae365c3dbf30657e2dc6449989e83d60000000000000000000000";
    let decoded_datalake =
        BlockSampledDatalake::from_serialized(blocksample_datalake.to_string()).unwrap();
    let block_datalake = BlockSampledDatalake::new(
        10399990,
        10400000,
        "account.0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6.nonce".to_string(),
        1,
    );
    assert_eq!(decoded_datalake, block_datalake);
}

#[test]
fn test_block_datalake_for_storage() {
    let blocksample_datalake = "0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009eb0f600000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000035037b2f05ce9ae365c3dbf30657e2dc6449989e83d600000000000000000000000000000000000000000000000000000000000000ff0000000000000000000000";
    let decoded_datalake =
        BlockSampledDatalake::from_serialized(blocksample_datalake.to_string()).unwrap();
    let block_datalake = BlockSampledDatalake::new(
        10399990,
        10400000,
        "storage.0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6.0x00000000000000000000000000000000000000000000000000000000000000ff".to_string(),
        1,
    );
    assert_eq!(decoded_datalake, block_datalake);
}

#[test]
fn test_blocksampled_header_compiler() {
    let block_datalake =
        BlockSampledDatalake::new(10399990, 10399995, "header.base_fee_per_gas".to_string(), 1);

    let data_points = block_datalake.compile().unwrap();
    assert_eq!(
        data_points,
        vec![
            DataPoint::Str("13".to_string()),
            DataPoint::Str("13".to_string()),
            DataPoint::Str("14".to_string()),
            DataPoint::Str("14".to_string()),
            DataPoint::Str("14".to_string()),
            DataPoint::Str("15".to_string()),
        ]
    );
}

#[test]
fn test_blocksampled_account_compiler() {
    let block_datalake = BlockSampledDatalake::new(
        10399990,
        10399992,
        "account.0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6.nonce".to_string(),
        1,
    );

    let data_points = block_datalake.compile().unwrap();
    assert_eq!(
        data_points,
        vec![
            DataPoint::Str("1".to_string()),
            DataPoint::Str("2".to_string()),
            DataPoint::Str("2".to_string()),
        ]
    );
}

#[test]
fn test_blocksampled_storage_compiler() {
    let block_datalake = BlockSampledDatalake::new(
        10399990,
        10399992,
        "storage.0x00000000000000adc04c56bf30ac9d3c0aaf14dc.0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
        1,
    );
    let data_points = block_datalake.compile().unwrap();
    assert_eq!(
        data_points,
        vec![
            DataPoint::Str(
                "0x0000000000000000000000000000000000000000000000000000000000000001".to_string()
            ),
            DataPoint::Str(
                "0x0000000000000000000000000000000000000000000000000000000000000002".to_string()
            ),
            DataPoint::Str(
                "0x0000000000000000000000000000000000000000000000000000000000000001".to_string()
            ),
        ]
    );
}
