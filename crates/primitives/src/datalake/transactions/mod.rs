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
        let encoded_datalake= "0x000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000f42400000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000";

        let transaction_datalake =
            TransactionsInBlockDatalake::new(1000000, "tx.nonce".to_string(), 1).unwrap();

        let encoded = transaction_datalake.encode().unwrap();

        assert_eq!(encoded, encoded_datalake);

        assert_eq!(
            transaction_datalake.commit(),
            "0xaec17f3d445f6c1b3147a512e6325bad3bb3337e229165e95c8d95dfa051b358"
        );

        assert_eq!(
            transaction_datalake.sampled_property,
            TransactionsCollection::Transactions(TransactionField::Nonce)
        );

        let decoded = TransactionsInBlockDatalake::decode(&encoded).unwrap();
        assert_eq!(decoded, transaction_datalake);
    }

    #[test]
    fn test_transactions_datalake_receipt() {
        let encoded_datalake = "0x000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000f42400000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000020100000000000000000000000000000000000000000000000000000000000000";
        let transaction_datalake =
            TransactionsInBlockDatalake::new(1000000, "tx_receipt.success".to_string(), 1).unwrap();

        let encoded = transaction_datalake.encode().unwrap();

        assert_eq!(encoded, encoded_datalake);

        assert_eq!(
            transaction_datalake.commit(),
            "0xcf3112eec35d7ab4205bccb353c76650e8eeed0e384c5e63d3b97c26402ffc4a"
        );

        assert_eq!(
            transaction_datalake.sampled_property,
            TransactionsCollection::TranasactionReceipts(TransactionReceiptField::Success)
        );

        let decoded = TransactionsInBlockDatalake::decode(&encoded).unwrap();
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
