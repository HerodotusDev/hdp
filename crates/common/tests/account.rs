use std::str::FromStr;

use alloy_primitives::{FixedBytes, U256};
use common::block::account::Account;

#[test]
fn test_get_account_rlp() {
    // let account_addr = "0x7b2f05ce9ae365c3dbf30657e2dc6449989e83d6";
    let account = Account::new(
        1,
        U256::from(0),
        FixedBytes::from_str("0x1c35dfde2b62d99d3a74fda76446b60962c4656814bdd7815eb6e5b8be1e7185")
            .unwrap(),
        FixedBytes::from_str("0xcd4f25236fff0ccac15e82bf4581beb08e95e1b5ba89de6031c75893cd91245c")
            .unwrap(),
    );
    let account_rlp = account.rlp_encode();
    assert_eq!(account_rlp, "f8440180a01c35dfde2b62d99d3a74fda76446b60962c4656814bdd7815eb6e5b8be1e7185a0cd4f25236fff0ccac15e82bf4581beb08e95e1b5ba89de6031c75893cd91245c");
    let account = Account::new(
        2,
        U256::from(0),
        FixedBytes::from_str("0x1c35dfde2b62d99d3a74fda76446b60962c4656814bdd7815eb6e5b8be1e7185")
            .unwrap(),
        FixedBytes::from_str("0xcd4f25236fff0ccac15e82bf4581beb08e95e1b5ba89de6031c75893cd91245c")
            .unwrap(),
    );
    let account_rlp = account.rlp_encode();
    assert_eq!(account_rlp, "f8440280a01c35dfde2b62d99d3a74fda76446b60962c4656814bdd7815eb6e5b8be1e7185a0cd4f25236fff0ccac15e82bf4581beb08e95e1b5ba89de6031c75893cd91245c");
    let account = Account::new(
        2,
        U256::from(0x1),
        FixedBytes::from_str("0x1c35dfde2b62d99d3a74fda76446b60962c4656814bdd7815eb6e5b8be1e7185")
            .unwrap(),
        FixedBytes::from_str("0xcd4f25236fff0ccac15e82bf4581beb08e95e1b5ba89de6031c75893cd91245c")
            .unwrap(),
    );
    let account_rlp = account.rlp_encode();
    assert_eq!(account_rlp, "f8440201a01c35dfde2b62d99d3a74fda76446b60962c4656814bdd7815eb6e5b8be1e7185a0cd4f25236fff0ccac15e82bf4581beb08e95e1b5ba89de6031c75893cd91245c");
}

#[test]
fn test_decode_account_rlp() {
    let account_rlp = "f8440180a01c35dfde2b62d99d3a74fda76446b60962c4656814bdd7815eb6e5b8be1e7185a0cd4f25236fff0ccac15e82bf4581beb08e95e1b5ba89de6031c75893cd91245c";
    let account = Account::rlp_decode(account_rlp);
    assert_eq!(
        account,
        Account::new(
            1,
            U256::from(0),
            FixedBytes::from_str(
                "0x1c35dfde2b62d99d3a74fda76446b60962c4656814bdd7815eb6e5b8be1e7185"
            )
            .unwrap(),
            FixedBytes::from_str(
                "0xcd4f25236fff0ccac15e82bf4581beb08e95e1b5ba89de6031c75893cd91245c"
            )
            .unwrap()
        )
    );
}
