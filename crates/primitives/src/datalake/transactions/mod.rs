pub mod collection;
pub mod datalake;
pub mod output;
pub mod rlp_fields;

// Export all types
pub use collection::*;
pub use datalake::*;
pub use rlp_fields::*;

#[cfg(test)]
mod tests {
    use crate::datalake::{Datalake, DatalakeCollection};

    use super::*;

    #[test]
    fn test_transactions_datalake() {
        let encoded_datalake= "0x0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000cb96aca8719987d15aecd066b7a1ad5d4d92fdd300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000";

        let transaction_datalake = TransactionsDatalake::new(
            "0xcb96AcA8719987D15aecd066B7a1Ad5D4d92fdD3".to_string(),
            0,
            3,
            "tx.nonce".to_string(),
            1,
        )
        .unwrap();

        let encoded = transaction_datalake.encode().unwrap();

        assert_eq!(encoded, encoded_datalake);

        assert_eq!(
            transaction_datalake.commit(),
            "0xc8faa3f58f9b18e0716c7c3358f47c3cc5701d71cdf7f4cc93a08afcb4397c5d"
        );

        assert_eq!(
            transaction_datalake.sampled_property,
            TransactionsCollection::Transactions(TransactionField::Nonce)
        );

        let decoded = TransactionsDatalake::decode(&encoded).unwrap();
        assert_eq!(decoded, transaction_datalake);
    }

    #[test]
    fn test_transactions_datalake_receipt() {
        let encoded_datalake = "0x0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000cb96aca8719987d15aecd066b7a1ad5d4d92fdd300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000000020100000000000000000000000000000000000000000000000000000000000000";
        let transaction_datalake = TransactionsDatalake::new(
            "0xcb96AcA8719987D15aecd066B7a1Ad5D4d92fdD3".to_string(),
            0,
            3,
            "tx_receipt.success".to_string(),
            1,
        )
        .unwrap();

        let encoded = transaction_datalake.encode().unwrap();

        assert_eq!(encoded, encoded_datalake);

        assert_eq!(
            transaction_datalake.commit(),
            "0xddb3e1df092aaf6ff5aa20e96df61d1f9110dca58dca5973349577ddcb86ec5e"
        );

        assert_eq!(
            transaction_datalake.sampled_property,
            TransactionsCollection::TranasactionReceipts(TransactionReceiptField::Success)
        );

        let decoded = TransactionsDatalake::decode(&encoded).unwrap();
        assert_eq!(decoded, transaction_datalake);
    }

    #[test]
    fn test_tx_collection_serialize() {
        let tx_collection = TransactionsCollection::Transactions(TransactionField::Nonce);
        let serialized = tx_collection.serialize().unwrap();
        assert_eq!(serialized, [0, 0]);

        let tx_collection =
            TransactionsCollection::TranasactionReceipts(TransactionReceiptField::Logs);
        let serialized = tx_collection.serialize().unwrap();
        assert_eq!(serialized, [1, 2]);

        let tx_collection = TransactionsCollection::Transactions(TransactionField::AccessList);
        let serialized = tx_collection.serialize().unwrap();
        assert_eq!(serialized, [0, 10]);
    }

    #[test]
    fn test_tx_collection_deserialize() {
        let serialized = [0, 1];
        let tx_collection = TransactionsCollection::deserialize(&serialized).unwrap();
        assert_eq!(
            tx_collection,
            TransactionsCollection::Transactions(TransactionField::GasPrice)
        );

        let serialized = [1, 3];
        let tx_collection = TransactionsCollection::deserialize(&serialized).unwrap();
        assert_eq!(
            tx_collection,
            TransactionsCollection::TranasactionReceipts(TransactionReceiptField::Bloom)
        );

        let serialized = [0, 10];
        let tx_collection = TransactionsCollection::deserialize(&serialized).unwrap();
        assert_eq!(
            tx_collection,
            TransactionsCollection::Transactions(TransactionField::AccessList)
        );
    }
}
