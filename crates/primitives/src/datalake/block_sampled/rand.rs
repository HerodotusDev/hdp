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
        let storage_index = rng.gen_range(0..5);
        let storage_key = B256::from(U256::from(storage_index));
        match index {
            0 => BlockSampledCollection::Header(self.sample(rng)),
            1 => BlockSampledCollection::Account(rand_account, self.sample(rng)),
            2 => BlockSampledCollection::Storage(rand_account, storage_key),
            _ => unreachable!(),
        }
    }
}

impl Distribution<HeaderField> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> HeaderField {
        let index: u8 = rng.gen_range(0..HeaderField::variants().len() as u8);
        HeaderField::from_index(index).unwrap()
    }
}

impl Distribution<AccountField> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> AccountField {
        let index: u8 = rng.gen_range(0..AccountField::variants().len() as u8);
        AccountField::from_index(index).unwrap()
    }
}
