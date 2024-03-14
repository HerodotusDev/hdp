#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use alloy_primitives::{hex, keccak256, FixedBytes, U256};

    use common::{
        block::{account::Account, header::BlockHeader},
        fetcher::{rpc::RpcFetcher, AbstractFetcher},
    };

    fn rlp_string_to_block_hash(rlp_string: &str) -> String {
        keccak256(hex::decode(rlp_string).unwrap()).to_string()
    }

    const GOERLI_RPC_URL: &str =
        "https://eth-goerli.g.alchemy.com/v2/OcJWF4RZDjyeCWGSmWChIlMEV28LtA5c";

    #[tokio::test]
    async fn test_rpc_get_block_by_number() {
        let fetcher = RpcFetcher::new(GOERLI_RPC_URL.into());

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
        let fetcher = RpcFetcher::new(GOERLI_RPC_URL.into());
        let target_address = "0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6".to_string();
        let account_from_rpc = fetcher
            .get_proof(10399990, target_address.clone(), None)
            .await
            .unwrap();
        let account: Account = Account::from(&account_from_rpc);
        let expected_account = Account::new(
            1,
            U256::from(0),
            FixedBytes::from_str(
                "0x480489b48e337887827fd9584f40dc1f51016e49df77ec789d4ee9bcc87bb0ff",
            )
            .unwrap(),
            FixedBytes::from_str(
                "0xcd4f25236fff0ccac15e82bf4581beb08e95e1b5ba89de6031c75893cd91245c",
            )
            .unwrap(),
        );
        assert_eq!(account, expected_account);
    }

    #[tokio::test]
    async fn test_fetcher_get_rlp_header() {
        let mut abstract_fetcher = AbstractFetcher::new(GOERLI_RPC_URL.into());
        let rlp_header = abstract_fetcher.get_rlp_header(0).await;
        let block_hash = rlp_string_to_block_hash(&rlp_header);
        assert_eq!(
            block_hash,
            "0xbf7e331f7f7c1dd2e05159666b3bf8bc7a8a3a9eb1d518969eab529dd9b88c1a"
        );
        let rlp_header = abstract_fetcher.get_rlp_header(10399990).await;
        let block_hash = rlp_string_to_block_hash(&rlp_header);
        assert_eq!(
            block_hash,
            "0x2ef5bd5264f472d821fb950241aa2bbe83f885fea086b4f58fccb9c9b948adcf"
        );
        let rlp_header = abstract_fetcher.get_rlp_header(487680).await;
        let block_hash = rlp_string_to_block_hash(&rlp_header);
        assert_eq!(
            block_hash,
            "0x9372b3057affe70c15a3a62dbdcb188677bdc8a403bc097acc22995544b27ba7"
        );
    }

    #[tokio::test]
    async fn test_fetcher_get_rlp_account() {
        let mut abstract_fetcher = AbstractFetcher::new(GOERLI_RPC_URL.into());
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

    const SEPOLIA_RPC_URL: &str =
        "https://eth-sepolia.g.alchemy.com/v2/a-w72ZvoUS0dfMD_LBPAuRzHOlQEhi_m";

    #[tokio::test]
    async fn test_fetcher_get_non_exist_storage_value() {
        let mut abstract_fetcher = AbstractFetcher::new(SEPOLIA_RPC_URL.into());
        let storage_value = abstract_fetcher
            .get_storage_value_with_proof(
                0,
                "0x75CeC1db9dCeb703200EAa6595f66885C962B920".to_string(),
                "0x0000000000000000000000000000000000000000000000000000000000000002".to_string(),
            )
            .await;

        assert!(storage_value.is_err());

        let storage_value = abstract_fetcher
            .get_storage_value_with_proof(
                20,
                "0x75CeC1db9dCeb703200EAa6595f66885C962B920".to_string(),
                "0x0000000000000000000000000000000000000000000000000000000000000002".to_string(),
            )
            .await;
        assert!(storage_value.is_err());

        // Actually the storage value is not 0x0 for later block, in the case, proof is not empty
        let storage_value = abstract_fetcher
            .get_storage_value_with_proof(
                5382810,
                "0x75CeC1db9dCeb703200EAa6595f66885C962B920".to_string(),
                "0x0000000000000000000000000000000000000000000000000000000000000002".to_string(),
            )
            .await;
        assert!(storage_value.is_ok());
        let storage_value = storage_value.unwrap();
        assert_eq!(storage_value.0, "0x9184e72a000");
        assert_eq!(storage_value.1, vec!["0xf8918080a0b7a7c859e6ddbad6c18adb60b9f48842e652021b4f8b875894b8b879568629f880a0e7f9c6d331c7d110c992550a7baa3e051adc1e26a53d928dbd517a313d221863808080808080a0e40cf9c20b1e8e4aaf3201dd3cb84ab06d2bac34e8dc3e918626e5c44c4f0707808080a0c01a2f302bfc71151daac60eeb4c1b73470845d4fe219e71644752abaafb02ab80", "0xe9a0305787fa12a823e0f2b7631cc41b3ba8828b3321ca811111fa75cd3aa3bb5ace878609184e72a000"]);

        // Even actually storage value is 0x0, but the proof is not empty
        let storage_value = abstract_fetcher
            .get_storage_value_with_proof(
                5382769,
                "0x75CeC1db9dCeb703200EAa6595f66885C962B920".to_string(),
                "0x0000000000000000000000000000000000000000000000000000000000000002".to_string(),
            )
            .await;
        assert!(storage_value.is_ok());
        let storage_value = storage_value.unwrap();
        assert_eq!(storage_value.0, "0x0");
        assert_eq!(storage_value.1, vec!["0xf838a120290decd9548b62a8d60345a988386fc84ba6bc95484008f6362f93160ef3e563959441ad2bc63a2059f9b623533d87fe99887d794847"]);
    }
}
