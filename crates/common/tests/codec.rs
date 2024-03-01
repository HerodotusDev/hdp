use common::codec::{datalakes_decoder, datalakes_encoder, tasks_decoder};
use common::datalake::{block_sampled::BlockSampledDatalake, Datalake};

#[test]
fn test_task_decoder() {
    // Note: all task's datalake is None
    let decoded_tasks = tasks_decoder("0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000060617667000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000006073756d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000606d696e00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000606d6178000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000000".to_string()).unwrap();
    assert_eq!(decoded_tasks.len(), 4);
    assert_eq!(decoded_tasks[0].aggregate_fn_id, "avg".to_string());
    assert_eq!(decoded_tasks[0].aggregate_fn_ctx, None);
    assert_eq!(decoded_tasks[1].aggregate_fn_id, "sum".to_string());
    assert_eq!(decoded_tasks[1].aggregate_fn_ctx, None);
    assert_eq!(decoded_tasks[2].aggregate_fn_id, "min".to_string());
    assert_eq!(decoded_tasks[2].aggregate_fn_ctx, None);
    assert_eq!(decoded_tasks[3].aggregate_fn_id, "max".to_string());
    assert_eq!(decoded_tasks[3].aggregate_fn_ctx, None);
}

#[test]
fn test_block_datalake_decoder() {
    let batched_block_datalake = "0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000280000000000000000000000000000000000000000000000000000000000000038000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009eb0f600000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009eb0f600000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009eb0f600000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009eb0f600000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f000000000000000000000000000000000000000000000000000000000000";
    let decoded_datalakes = datalakes_decoder(batched_block_datalake.to_string()).unwrap();
    assert_eq!(decoded_datalakes.len(), 4);
    for datalake in decoded_datalakes.clone() {
        if let Datalake::BlockSampled(block_datalake) = datalake {
            assert_eq!(block_datalake.block_range_start, 10399990);
            assert_eq!(block_datalake.block_range_end, 10400000);
            assert_eq!(
                block_datalake.sampled_property,
                "header.base_fee_per_gas".to_string()
            );
            assert_eq!(block_datalake.increment, 1);
        } else {
            panic!("Expected block datalake");
        }
    }

    assert_eq!(
        datalakes_encoder(decoded_datalakes).unwrap(),
        batched_block_datalake
    );
}

#[test]
fn test_block_datalake_decoder_for_account() {
    let batched_block_datalake = "0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004b902400000000000000000000000000000000000000000000000000000000004b9027000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000016020a4de450feb156a2a51ed159b2fb99da26e5f3a30000000000000000000000";
    let block_datalake = BlockSampledDatalake::new(
        4952100,
        4952103,
        "account.0x0a4de450feb156a2a51ed159b2fb99da26e5f3a3.nonce".to_string(),
        1,
    );
    let datalakes = vec![Datalake::BlockSampled(block_datalake.clone())];
    assert_eq!(datalakes.len(), 1);
    for datalake in datalakes.clone() {
        if let Datalake::BlockSampled(block_datalake) = datalake {
            assert_eq!(block_datalake.block_range_start, 4952100);
            assert_eq!(block_datalake.block_range_end, 4952103);
            assert_eq!(
                block_datalake.sampled_property,
                "account.0x0a4de450feb156a2a51ed159b2fb99da26e5f3a3.nonce".to_string()
            );
            assert_eq!(block_datalake.increment, 1);
        } else {
            panic!("Expected block datalake");
        }
    }

    assert_eq!(
        datalakes_encoder(datalakes.clone()).unwrap(),
        batched_block_datalake
    );
    assert_eq!(
        datalakes_decoder(batched_block_datalake.to_string()).unwrap(),
        datalakes
    );
}

#[test]
fn test_block_massive_datalake_decoder() {
    let batched_block_datalake = "0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000280000000000000000000000000000000000000000000000000000000000000038000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009ead1800000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009ead1800000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009ead1800000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000009ead1800000000000000000000000000000000000000000000000000000000009eb100000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000002010f000000000000000000000000000000000000000000000000000000000000";
    let datalake_massive_block = Datalake::BlockSampled(BlockSampledDatalake::new(
        10399000,
        10400000,
        "header.base_fee_per_gas".to_string(),
        1,
    ));

    let batched_datalakes = vec![
        datalake_massive_block.clone(),
        datalake_massive_block.clone(),
        datalake_massive_block.clone(),
        datalake_massive_block.clone(),
    ];
    let decoded_datalakes = datalakes_decoder(batched_block_datalake.to_string()).unwrap();
    assert_eq!(decoded_datalakes.len(), 4);

    assert_eq!(
        datalakes_encoder(batched_datalakes).unwrap(),
        batched_block_datalake
    );
}

#[test]
fn test_dynamic_layout_datalake_decoder() {
    let batched_dynamic_layer_datalake = "0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000280000000000000000000000000000000000000000000000000000000000000038000000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000009eb0f60000000000000000000000007b2f05ce9ae365c3dbf30657e2dc6449989e83d6000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000009eb0f60000000000000000000000007b2f05ce9ae365c3dbf30657e2dc6449989e83d6000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000009eb0f60000000000000000000000007b2f05ce9ae365c3dbf30657e2dc6449989e83d6000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000009eb0f60000000000000000000000007b2f05ce9ae365c3dbf30657e2dc6449989e83d60000000000000000000000000000000000000000000000000000000000000005000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000001";
    let decoded_datalakes = datalakes_decoder(batched_dynamic_layer_datalake.to_string()).unwrap();
    assert_eq!(decoded_datalakes.len(), 4);
    for datalake in decoded_datalakes {
        if let Datalake::DynamicLayout(dynamic_datalake) = datalake {
            assert_eq!(dynamic_datalake.block_number, 10399990);
            assert_eq!(
                dynamic_datalake.account_address,
                "0x7b2f05cE9aE365c3DBF30657e2DC6449989e83D6".to_string()
            );
            assert_eq!(dynamic_datalake.slot_index, 5);
            assert_eq!(dynamic_datalake.initial_key, 0);
            assert_eq!(dynamic_datalake.key_boundry, 3);
            assert_eq!(dynamic_datalake.increment, 1);
        } else {
            panic!("Expected dynamic layout datalake");
        }
    }
}