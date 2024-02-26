use std::str::FromStr;

use alloy_primitives::{FixedBytes, U256};
use common::{
    block::{account::Account, header::BlockHeader},
    fetcher::{rpc::RpcFetcher, AbstractFetcher},
    utils::rlp_string_to_block_hash,
};

#[tokio::test]
async fn test_rpc_get_block_by_number() {
    let fetcher = RpcFetcher::new("https://ethereum-goerli.publicnode.com".to_string());

    let block = fetcher.get_block_by_number(0).await.unwrap();
    let block_header = BlockHeader::from(&block);
    assert_eq!(block.get_block_hash(), block_header.get_block_hash());

    let block = fetcher.get_block_by_number(10487680).await.unwrap();
    let block_header = BlockHeader::from(&block);
    assert_eq!(block.get_block_hash(), block_header.get_block_hash());

    let block = fetcher.get_block_by_number(487680).await.unwrap();
    let block_header = BlockHeader::from(&block);
    assert_eq!(block.get_block_hash(), block_header.get_block_hash());
}

#[tokio::test]
async fn test_rpc_get_proof() {
    let fetcher = RpcFetcher::new(
        "https://eth-goerli.g.alchemy.com/v2/OcJWF4RZDjyeCWGSmWChIlMEV28LtA5c".to_string(),
    );
    let target_address = "0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6".to_string();
    let account_from_rpc = fetcher
        .get_proof(10399990, target_address.clone(), None)
        .await
        .unwrap();
    let account: Account = Account::from(&account_from_rpc);
    let expected_account = Account::new(
        1,
        U256::from(0),
        FixedBytes::from_str("0x480489b48e337887827fd9584f40dc1f51016e49df77ec789d4ee9bcc87bb0ff")
            .unwrap(),
        FixedBytes::from_str("0xcd4f25236fff0ccac15e82bf4581beb08e95e1b5ba89de6031c75893cd91245c")
            .unwrap(),
    );
    assert_eq!(account, expected_account);
}

#[tokio::test]
async fn test_fetcher_get_rlp_header() {
    let mut abstract_fetcher = AbstractFetcher::new(
        "https://eth-goerli.g.alchemy.com/v2/OcJWF4RZDjyeCWGSmWChIlMEV28LtA5c".to_string(),
    );
    let rlp_header = abstract_fetcher.get_rlp_header(0).await;
    let block_hash = rlp_string_to_block_hash(&rlp_header).unwrap();
    assert_eq!(
        block_hash,
        "0xbf7e331f7f7c1dd2e05159666b3bf8bc7a8a3a9eb1d518969eab529dd9b88c1a"
    );
    let rlp_header = abstract_fetcher.get_rlp_header(10399990).await;
    let block_hash = rlp_string_to_block_hash(&rlp_header).unwrap();
    assert_eq!(
        block_hash,
        "0x2ef5bd5264f472d821fb950241aa2bbe83f885fea086b4f58fccb9c9b948adcf"
    );
    let rlp_header = abstract_fetcher.get_rlp_header(487680).await;
    let block_hash = rlp_string_to_block_hash(&rlp_header).unwrap();
    assert_eq!(
        block_hash,
        "0x9372b3057affe70c15a3a62dbdcb188677bdc8a403bc097acc22995544b27ba7"
    );
}

#[tokio::test]
async fn test_fetcher_get_rlp_account() {
    let mut abstract_fetcher = AbstractFetcher::new(
        "https://eth-goerli.g.alchemy.com/v2/OcJWF4RZDjyeCWGSmWChIlMEV28LtA5c".to_string(),
    );
    let rlp_account = abstract_fetcher
        .get_account_with_proof(0, "0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6".to_string())
        .await;
    assert_eq!(rlp_account.0, "f8448080a00000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000000");
    let rlp_account = abstract_fetcher
        .get_account_with_proof(
            10399990,
            "0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6".to_string(),
        )
        .await;
    assert_eq!(rlp_account.0, "f8440180a0480489b48e337887827fd9584f40dc1f51016e49df77ec789d4ee9bcc87bb0ffa0cd4f25236fff0ccac15e82bf4581beb08e95e1b5ba89de6031c75893cd91245c");
}

#[tokio::test]
async fn test_fetcher_get_storage_value() {
    let mut abstract_fetcher = AbstractFetcher::new(
        "https://eth-goerli.g.alchemy.com/v2/OcJWF4RZDjyeCWGSmWChIlMEV28LtA5c".to_string(),
    );
    let storage_value = abstract_fetcher
        .get_storage_value_with_proof(
            0,
            "0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6".to_string(),
            "0x0".to_string(),
        )
        .await;
    assert_eq!(storage_value.0, "0x0");
    let storage_value = abstract_fetcher
        .get_storage_value_with_proof(
            10399990,
            "0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6".to_string(),
            "0x0".to_string(),
        )
        .await;
    assert_eq!(storage_value.0, "0x1");
    let storage_value = abstract_fetcher
        .get_storage_value_with_proof(
            10399980,
            "0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6".to_string(),
            "0x0".to_string(),
        )
        .await;
    assert_eq!(storage_value.0, "0x1");
}
