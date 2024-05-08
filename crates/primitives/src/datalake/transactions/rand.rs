use rand::distributions::{Distribution, Standard};

use crate::datalake::DatalakeField;

use super::{TransactionField, TransactionReceiptField, TransactionsCollection};

impl Distribution<TransactionsCollection> for Standard {
    fn sample<R: rand::prelude::Rng + ?Sized>(&self, rng: &mut R) -> TransactionsCollection {
        // TODO: For now receipt doesn't support in Cairo
        let index: u8 = rng.gen_range(0..1);
        match index {
            0 => TransactionsCollection::Transactions(self.sample(rng)),
            1 => TransactionsCollection::TranasactionReceipts(self.sample(rng)),
            _ => unreachable!(),
        }
    }
}

impl Distribution<TransactionField> for Standard {
    fn sample<R: rand::prelude::Rng + ?Sized>(&self, rng: &mut R) -> TransactionField {
        let index: u8 = rng.gen_range(0..=6_u8);
        TransactionField::integer_variants_index(index)
    }
}

impl Distribution<TransactionReceiptField> for Standard {
    fn sample<R: rand::prelude::Rng + ?Sized>(&self, rng: &mut R) -> TransactionReceiptField {
        let index: u8 = rng.gen_range(0..=1_u8);
        TransactionReceiptField::from_index(index).unwrap()
    }
}
