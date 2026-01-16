use std::fmt::Display;

use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Clone, Debug, PartialEq)]
pub enum Transaction {
    Deposit(u16, u32, Decimal),
    Withdrawal(u16, u32, Decimal),
    Dispute(u16, u32),
    Resolve(u16, u32),
    Chargeback(u16, u32)
}

#[derive(Debug)]
pub enum TransactionError {
    UnknownType,
}

impl Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionError::UnknownType => write!(f, "Unknown transaction type"),
        }
    }
}

impl Transaction {
    pub fn new(ttype: &str, client: u16, tx: u32, amount: Decimal) -> Result<Self, TransactionError> {
        match ttype {
            "deposit" => Ok(Transaction::Deposit(client, tx, amount)),
            "withdrawal" => Ok(Transaction::Withdrawal(client, tx, amount)),
            "dispute" => Ok(Transaction::Dispute(client, tx)),
            "resolve" => Ok(Transaction::Resolve(client, tx)),
            "chargeback" => Ok(Transaction::Chargeback(client, tx)),
            _ => Err(TransactionError::UnknownType),
        }
    }
}

impl<'de> Deserialize<'de> for Transaction {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct TransactionRecord {
            ttype: String,
            client: u16,
            tx: u32,
            amount: Option<Decimal>,
        }
        let record = TransactionRecord::deserialize(deserializer)?;
        let amount = record.amount.unwrap_or(Decimal::ZERO).round_dp(4);
        Transaction::new(&record.ttype, record.client, record.tx, amount)
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deposit_transaction_creation() {
        let amount = Decimal::new(100000, 4); // 10.00
        let tx = Transaction::new("deposit", 1, 100, amount);
        assert!(tx.is_ok());
        match tx.unwrap() {
            Transaction::Deposit(client, tx_id, amt) => {
                assert_eq!(client, 1);
                assert_eq!(tx_id, 100);
                assert_eq!(amt, amount);
            }
            _ => panic!("Expected Deposit transaction"),
        }
    }

    #[test]
    fn test_withdrawal_transaction_creation() {
        let amount = Decimal::new(50000, 4); // 5.00
        let tx = Transaction::new("withdrawal", 2, 101, amount);
        assert!(tx.is_ok());
        match tx.unwrap() {
            Transaction::Withdrawal(client, tx_id, amt) => {
                assert_eq!(client, 2);
                assert_eq!(tx_id, 101);
                assert_eq!(amt, amount);
            }
            _ => panic!("Expected Withdrawal transaction"),
        }
    }

    #[test]
    fn test_dispute_transaction_creation() {
        let tx = Transaction::new("dispute", 3, 102, Decimal::ZERO);
        assert!(tx.is_ok());
        match tx.unwrap() {
            Transaction::Dispute(client, tx_id) => {
                assert_eq!(client, 3);
                assert_eq!(tx_id, 102);
            }
            _ => panic!("Expected Dispute transaction"),
        }
    }

    #[test]
    fn test_resolve_transaction_creation() {
        let tx = Transaction::new("resolve", 4, 103, Decimal::ZERO);
        assert!(tx.is_ok());
        match tx.unwrap() {
            Transaction::Resolve(client, tx_id) => {
                assert_eq!(client, 4);
                assert_eq!(tx_id, 103);
            }
            _ => panic!("Expected Resolve transaction"),
        }
    }

    #[test]
    fn test_chargeback_transaction_creation() {
        let tx = Transaction::new("chargeback", 5, 104, Decimal::ZERO);
        assert!(tx.is_ok());
        match tx.unwrap() {
            Transaction::Chargeback(client, tx_id) => {
                assert_eq!(client, 5);
                assert_eq!(tx_id, 104);
            }
            _ => panic!("Expected Chargeback transaction"),
        }
    }

    #[test]
    fn test_unknown_transaction_type() {
        let amount = Decimal::new(100, 4);
        let result = Transaction::new("unknown", 6, 105, amount);
        assert!(result.is_err());
    }

    #[test]
    fn test_transaction_deserialization() {
        let csv_data = "ttype,client,tx,amount
deposit,1,100,10.00
withdrawal,2,101,5.123456789
dispute,3,102,
resolve,4,103,
chargeback,5,104,";

        let mut reader = csv::Reader::from_reader(csv_data.as_bytes());

        let transactions = reader.records().map(|rec| {
            let record = rec.unwrap();
            record.deserialize(None).unwrap()
        }).collect::<Vec<Transaction>>();
        assert_eq!(transactions.len(), 5);
        assert_eq!(transactions[0], Transaction::Deposit(1, 100, Decimal::new(100000, 4)));
        assert_eq!(transactions[1], Transaction::Withdrawal(2, 101, Decimal::new(51235, 4)));
        assert_eq!(transactions[2], Transaction::Dispute(3, 102));
        assert_eq!(transactions[3], Transaction::Resolve(4, 103));
        assert_eq!(transactions[4], Transaction::Chargeback(5, 104));
    }
}