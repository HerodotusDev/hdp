pub mod collection;
pub mod datalake;
pub mod output;
pub mod rand;
pub mod rlp_fields;

// Export all types
pub use collection::*;
pub use datalake::*;
pub use rlp_fields::*;

#[cfg(test)]
mod tests {

    use crate::datalake::{Datalake, DatalakeCollection};
    use alloy_primitives::U256;

    use super::*;

    #[test]
    fn test_transactions_datalake() {
        let encoded_datalake= "0x000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000f42400000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000101010100000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000000020100000000000000000000000000000000000000000000000000000000000000";

        let transaction_datalake = TransactionsInBlockDatalake::new(
            1000000,
            "tx.nonce".to_string(),
            1,
            10,
            2,
            &[1, 1, 1, 1],
        )
        .unwrap();

        let encoded = transaction_datalake.encode().unwrap();

        assert_eq!(encoded, encoded_datalake);

        assert_eq!(
            transaction_datalake.commit(),
            "0xbcc6e9f8aea4122dedf75f9cc0ce1126b3a38694366ee6f98ddbfffffeaf8e1a"
        );

        assert_eq!(
            transaction_datalake.sampled_property,
            TransactionsCollection::Transactions(TransactionField::Nonce)
        );

        assert_eq!(
            transaction_datalake.included_types.to_uint256(),
            U256::from(0x01010101)
        );

        let decoded = TransactionsInBlockDatalake::decode(&encoded).unwrap();
        assert_eq!(decoded, transaction_datalake);
    }

    #[test]
    fn test_transactions_datalake_receipt() {
        let encoded_datalake = "0x000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000f42400000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000100000100000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000000020200000000000000000000000000000000000000000000000000000000000000";
        let transaction_datalake = TransactionsInBlockDatalake::new(
            1000000,
            "tx_receipt.success".to_string(),
            1,
            10,
            2,
            &[1, 0, 0, 1],
        )
        .unwrap();

        let encoded = transaction_datalake.encode().unwrap();

        assert_eq!(encoded, encoded_datalake);

        assert_eq!(
            transaction_datalake.commit(),
            "0x64882f901df552341bc9582ae597945cc69575b8e37cde61eaacb3b0f7af682c"
        );

        assert_eq!(
            transaction_datalake.sampled_property,
            TransactionsCollection::TranasactionReceipts(TransactionReceiptField::Success)
        );

        assert_eq!(
            transaction_datalake.included_types.to_uint256(),
            U256::from(0x01000001)
        );

        let decoded = TransactionsInBlockDatalake::decode(&encoded).unwrap();
        assert_eq!(decoded, transaction_datalake);
    }

    #[test]
    fn test_tx_collection_serialize() {
        let tx_collection = TransactionsCollection::Transactions(TransactionField::Nonce);
        let serialized = tx_collection.serialize().unwrap();
        assert_eq!(serialized, [1, 0]);

        let tx_collection =
            TransactionsCollection::TranasactionReceipts(TransactionReceiptField::Logs);
        let serialized = tx_collection.serialize().unwrap();
        assert_eq!(serialized, [2, 2]);

        let tx_collection = TransactionsCollection::Transactions(TransactionField::AccessList);
        let serialized = tx_collection.serialize().unwrap();
        assert_eq!(serialized, [1, 10]);
    }

    #[test]
    fn test_tx_collection_deserialize() {
        let serialized = [1, 1];
        let tx_collection = TransactionsCollection::deserialize(&serialized).unwrap();
        assert_eq!(
            tx_collection,
            TransactionsCollection::Transactions(TransactionField::GasPrice)
        );

        let serialized = [2, 3];
        let tx_collection = TransactionsCollection::deserialize(&serialized).unwrap();
        assert_eq!(
            tx_collection,
            TransactionsCollection::TranasactionReceipts(TransactionReceiptField::Bloom)
        );

        let serialized = [1, 10];
        let tx_collection = TransactionsCollection::deserialize(&serialized).unwrap();
        assert_eq!(
            tx_collection,
            TransactionsCollection::Transactions(TransactionField::AccessList)
        );
    }
}
