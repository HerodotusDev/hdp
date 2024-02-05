#[tokio::test]
async fn test_rpc() {
    use common::rpc::RpcFetcher;

    let fetcher = RpcFetcher::new("https://ethereum-goerli.publicnode.com".to_string());
    let blocknumber = fetcher.get_block_by_number(0).await;
    println!("blocknumber: {:?}", blocknumber);
}
