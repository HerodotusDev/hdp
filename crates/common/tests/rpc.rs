use std::str::FromStr;

use alloy_primitives::{FixedBytes, U256};
use common::{block::account::Account, rpc::RpcFetcher};
use reth_primitives::Header;

#[tokio::test]
async fn test_rpc_get_block_by_number() {
    let fetcher = RpcFetcher::new("https://ethereum-goerli.publicnode.com".to_string());

    let block = fetcher.get_block_by_number(0).await.unwrap();
    let block_header = Header::from(&block);
    assert_eq!(block.get_block_hash(), block_header.hash_slow().to_string());

    let block = fetcher.get_block_by_number(10487680).await.unwrap();
    let block_header = Header::from(&block);
    assert_eq!(block.get_block_hash(), block_header.hash_slow().to_string());

    let block = fetcher.get_block_by_number(487680).await.unwrap();
    let block_header = Header::from(&block);
    assert_eq!(block.get_block_hash(), block_header.hash_slow().to_string());
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
