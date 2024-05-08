use std::str::FromStr;

use alloy_primitives::{Address, B256, U256};
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

use crate::datalake::DatalakeField;

use super::{AccountField, BlockSampledCollection, HeaderField};

impl Distribution<BlockSampledCollection> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> BlockSampledCollection {
        let index: u8 = rng.gen_range(0..3);
        let rand_account = Address::from_str("0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4").unwrap();
        let rand_account_for_storage =
            Address::from_str("0x75CeC1db9dCeb703200EAa6595f66885C962B920").unwrap();
        let storage_index = rng.gen_range(0..5);
        let storage_key = B256::from(U256::from(storage_index));
        match index {
            0 => BlockSampledCollection::Header(self.sample(rng)),
            1 => BlockSampledCollection::Account(rand_account, self.sample(rng)),
            2 => BlockSampledCollection::Storage(rand_account_for_storage, storage_key),
            _ => unreachable!(),
        }
    }
}

impl Distribution<HeaderField> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> HeaderField {
        let index = rng.gen_range(0..=8);
        HeaderField::integer_variants_index(index)
    }
}

impl Distribution<AccountField> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> AccountField {
        let index: u8 = rng.gen_range(0..=1_u8);
        AccountField::from_index(index).unwrap()
    }
}
