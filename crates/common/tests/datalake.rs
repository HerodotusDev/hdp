use common::datalake::{
    block_sampled::BlockSampledDatalake, dynamic_layout::DynamicLayoutDatalake,
};

#[test]
fn test_block_datalake_for_header() {
    let blocksample_datalake = "0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009eb0f600000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f000000000000000000000000000000000000000000000000000000000000";
    let decoded_datalake =
        BlockSampledDatalake::deserialize(blocksample_datalake.to_string()).unwrap();
    let block_datalake =
        BlockSampledDatalake::new(10399990, 10400000, "header.base_fee_per_gas".to_string(), 1);
    assert_eq!(
        decoded_datalake.serialize().unwrap(),
        block_datalake.serialize().unwrap()
    );

    assert_eq!(decoded_datalake.serialize().unwrap(), blocksample_datalake);
}

#[test]
fn test_block_datalake_for_header_massive() {
    let blocksample_datalake = "0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009d2a6000000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f000000000000000000000000000000000000000000000000000000000000";
    let decoded_datalake: BlockSampledDatalake =
        BlockSampledDatalake::deserialize(blocksample_datalake.to_string()).unwrap();
    let block_datalake =
        BlockSampledDatalake::new(10300000, 10400000, "header.base_fee_per_gas".to_string(), 1);
    assert_eq!(
        decoded_datalake.serialize().unwrap(),
        block_datalake.serialize().unwrap()
    );

    assert_eq!(decoded_datalake.serialize().unwrap(), blocksample_datalake);
}

#[test]
fn test_block_datalake_for_account() {
    let blocksample_datalake = "0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009eb0f600000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000016027b2f05ce9ae365c3dbf30657e2dc6449989e83d60000000000000000000000";
    let decoded_datalake =
        BlockSampledDatalake::deserialize(blocksample_datalake.to_string()).unwrap();
    let block_datalake = BlockSampledDatalake::new(
        10399990,
        10400000,
        "account.0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6.nonce".to_string(),
        1,
    );
    assert_eq!(decoded_datalake, block_datalake);
    assert_eq!(decoded_datalake.serialize().unwrap(), blocksample_datalake);
}

#[test]
fn test_block_datalake_for_storage() {
    let blocksample_datalake = "0x000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009eb0f600000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000035037b2f05ce9ae365c3dbf30657e2dc6449989e83d600000000000000000000000000000000000000000000000000000000000000ff0000000000000000000000";
    let decoded_datalake =
        BlockSampledDatalake::deserialize(blocksample_datalake.to_string()).unwrap();
    let block_datalake = BlockSampledDatalake::new(
        10399990,
        10400000,
        "storage.0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6.0x00000000000000000000000000000000000000000000000000000000000000ff".to_string(),
        1,
    );
    assert_eq!(decoded_datalake, block_datalake);
    assert_eq!(decoded_datalake.serialize().unwrap(), blocksample_datalake);
}

#[test]
fn test_dynamic_layout_datalake_serialized() {
    let dynamic_layout_datalake = "0x000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000009eb0f60000000000000000000000007b2f05ce9ae365c3dbf30657e2dc6449989e83d60000000000000000000000000000000000000000000000000000000000000005000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000001";
    let decoded_datalake =
        DynamicLayoutDatalake::deserialize(dynamic_layout_datalake.to_string()).unwrap();
    let dynamic_layout_datalake = DynamicLayoutDatalake::new(
        10399990,
        "0x7b2f05cE9aE365c3DBF30657e2DC6449989e83D6".to_string(),
        5,
        0,
        3,
        1,
    );
    assert_eq!(decoded_datalake, dynamic_layout_datalake);
    assert_eq!(
        decoded_datalake.serialize().unwrap(),
        dynamic_layout_datalake.serialize().unwrap()
    );
}