use std::{
    collections::{BTreeMap, BTreeSet},
    io,
};

use csv::Writer;
use rust_decimal::Decimal;

use crate::{client::Client, transaction::Transaction};

pub(crate) struct Engine {
    clients: BTreeMap<u16, Client>,
    transaction_log: BTreeMap<u32, Transaction>,
    disputed_transactions: BTreeSet<u32>,
}

#[derive(Debug, PartialEq)]
pub enum ExecutionError {
    InsufficientFunds,
    AccountLocked,
    TransactionNotFound,
    IneligibleTransaction,
    NonDisputedTransaction,
    AlreadyDisputedTransaction,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            clients: BTreeMap::new(),
            transaction_log: BTreeMap::new(),
            disputed_transactions: BTreeSet::new(),
        }
    }

    pub fn execute(&mut self, transaction: Transaction) -> Result<(), ExecutionError> {
        match transaction {
            Transaction::Deposit(client_id, tx_id, amount) => {
                let client = self.fetch_or_create_client_mut(client_id)?;
                client.available += amount;
                client.total += amount;
                // Logging only deposits and withdrawals
                self.transaction_log.insert(tx_id, transaction);
            }
            Transaction::Withdrawal(client_id, tx_id, amount) => {
                let client = self.fetch_or_create_client_mut(client_id)?;
                if client.available >= amount {
                    client.available -= amount;
                    client.total -= amount;
                    // Logging only deposits and withdrawals
                    self.transaction_log.insert(tx_id, transaction);
                } else {
                    return Err(ExecutionError::InsufficientFunds);
                }
            }
            Transaction::Dispute(_, tx_id) => {
                if self.disputed_transactions.contains(&tx_id) {
                    return Err(ExecutionError::AlreadyDisputedTransaction);
                }
                let (src_client_id, src_amount) = self.fetch_disputed_transaction(tx_id)?;
                let client = self.fetch_or_create_client_mut(src_client_id)?;
                client.available -= src_amount;
                client.held += src_amount;
                self.disputed_transactions.insert(tx_id);
            }
            Transaction::Resolve(_, tx_id) => {
                if !self.disputed_transactions.contains(&tx_id) {
                    return Err(ExecutionError::NonDisputedTransaction);
                }
                let (src_client_id, src_amount) = self.fetch_disputed_transaction(tx_id)?;
                let client = self.fetch_or_create_client_mut(src_client_id)?;
                client.available += src_amount;
                client.held -= src_amount;
                self.disputed_transactions.remove(&tx_id);
            }
            Transaction::Chargeback(_, tx_id) => {
                if !self.disputed_transactions.contains(&tx_id) {
                    return Err(ExecutionError::NonDisputedTransaction);
                }
                let (src_client_id, src_amount) = self.fetch_disputed_transaction(tx_id)?;
                let client = self.fetch_or_create_client_mut(src_client_id)?;
                client.held -= src_amount;
                client.total -= src_amount;
                client.locked = true;
                self.disputed_transactions.remove(&tx_id);
            }
        }
        Ok(())
    }

    fn fetch_or_create_client_mut(
        &mut self,
        client_id: u16,
    ) -> Result<&mut Client, ExecutionError> {
        let client = self
            .clients
            .entry(client_id)
            .or_insert(Client::new(client_id));
        if client.locked {
            return Err(ExecutionError::AccountLocked);
        }
        Ok(client)
    }

    pub fn print_client_report(&self) {
        let mut writer = Writer::from_writer(io::stdout());

        // Write header
        writer.write_record(&["client", "available", "held", "total", "locked"])
            .expect("failed to write CSV header");

        // Write rows
        for client in self.clients.values() {
            writer.write_record(&[
                client.id.to_string(),
                client.available.to_string(),
                client.held.to_string(),
                client.total.to_string(),
                client.locked.to_string(),
            ])
            .expect("failed to write CSV record");
        }

        // Ensure all data is flushed
        writer.flush().expect("failed to flush CSV writer");
    }

    fn fetch_disputed_transaction(&self, tx_id: u32) -> Result<(u16, Decimal), ExecutionError> {
        let transaction = self
            .transaction_log
            .get(&tx_id)
            .ok_or(ExecutionError::TransactionNotFound)?;
        match transaction {
            Transaction::Deposit(client_id, _, amount) => Ok((*client_id, *amount)),
            _ => Err(ExecutionError::IneligibleTransaction),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let engine = Engine::new();
        assert!(engine.clients.is_empty());
    }

    #[test]
    fn test_execution_deposit_and_withdraw() {
        let mut engine = Engine::new();
        let deposit = Transaction::Deposit(1, 100, Decimal::new(100000, 4));
        assert!(engine.execute(deposit).is_ok());
        {
            let client1 = engine.clients.get(&1).unwrap();
            assert_eq!(client1.available, Decimal::new(100000, 4));
            assert_eq!(client1.total, Decimal::new(100000, 4));
            assert!(!client1.locked);
        }
        let withdrawal = Transaction::Withdrawal(1, 101, Decimal::new(50000, 4));
        assert!(engine.execute(withdrawal).is_ok());
        {
            let client1 = engine.clients.get(&1).unwrap();
            assert_eq!(client1.available, Decimal::new(50000, 4));
            assert_eq!(client1.total, Decimal::new(50000, 4));
            assert!(!client1.locked);
        }
    }

    #[test]
    fn test_execution_deposit_and_resolved() {
        let mut engine = Engine::new();
        let deposit = Transaction::Deposit(1, 100, Decimal::new(100000, 4));
        assert!(engine.execute(deposit).is_ok());
        {
            let client1 = engine.clients.get(&1).unwrap();
            assert_eq!(client1.available, Decimal::new(100000, 4));
            assert_eq!(client1.total, Decimal::new(100000, 4));
            assert!(!client1.locked);
        }
        let dispute = Transaction::Dispute(1, 100);
        assert!(engine.execute(dispute).is_ok());
        {
            let client1 = engine.clients.get(&1).unwrap();
            assert_eq!(client1.available, Decimal::new(0, 4));
            assert_eq!(client1.held, Decimal::new(100000, 4));
            assert_eq!(client1.total, Decimal::new(100000, 4));
            assert!(!client1.locked);
        }
        let resolve = Transaction::Resolve(1, 100);
        assert!(engine.execute(resolve).is_ok());
        {
            let client1 = engine.clients.get(&1).unwrap();
            assert_eq!(client1.available, Decimal::new(100000, 4));
            assert_eq!(client1.held, Decimal::new(0, 4));
            assert_eq!(client1.total, Decimal::new(100000, 4));
            assert!(!client1.locked);
        }
    }

    #[test]
    fn test_execution_deposit_and_chargeback() {
        let mut engine = Engine::new();
        let deposit = Transaction::Deposit(1, 100, Decimal::new(100000, 4));
        assert!(engine.execute(deposit).is_ok());
        {
            let client1 = engine.clients.get(&1).unwrap();
            assert_eq!(client1.available, Decimal::new(100000, 4));
            assert_eq!(client1.total, Decimal::new(100000, 4));
            assert!(!client1.locked);
        }
        let dispute = Transaction::Dispute(1, 100);
        assert!(engine.execute(dispute).is_ok());
        {
            let client1 = engine.clients.get(&1).unwrap();
            assert_eq!(client1.available, Decimal::new(0, 4));
            assert_eq!(client1.held, Decimal::new(100000, 4));
            assert_eq!(client1.total, Decimal::new(100000, 4));
            assert!(!client1.locked);
        }
        let resolve = Transaction::Chargeback(1, 100);
        assert!(engine.execute(resolve).is_ok());
        {
            let client1 = engine.clients.get(&1).unwrap();
            assert_eq!(client1.available, Decimal::new(0, 4));
            assert_eq!(client1.held, Decimal::new(0, 4));
            assert_eq!(client1.total, Decimal::new(0, 4));
            assert!(client1.locked);
        }
        let deposit_after_lock = Transaction::Deposit(1, 101, Decimal::new(50000, 4));
        assert_eq!(
            engine.execute(deposit_after_lock).err(),
            Some(ExecutionError::AccountLocked)
        );
        let withdraw_after_lock = Transaction::Withdrawal(1, 102, Decimal::new(50000, 4));
        assert_eq!(
            engine.execute(withdraw_after_lock).err(),
            Some(ExecutionError::AccountLocked)
        );
    }

    #[test]
    fn test_execution_non_disputed_transaction() {
        let mut engine = Engine::new();
        let deposit = Transaction::Deposit(1, 100, Decimal::new(100000, 4));
        assert!(engine.execute(deposit).is_ok());
        let resolve = Transaction::Resolve(1, 100);
        assert_eq!(
            engine.execute(resolve).err(),
            Some(ExecutionError::NonDisputedTransaction)
        );
        let chargeback = Transaction::Chargeback(1, 100);
        assert_eq!(
            engine.execute(chargeback).err(),
            Some(ExecutionError::NonDisputedTransaction)
        );
    }

    #[test]
    fn test_execution_dispute_ineligible_transaction() {
        let mut engine = Engine::new();
        assert!(engine.execute(Transaction::Deposit(1, 100, Decimal::new(100000, 4))).is_ok());
        let withdrawal = Transaction::Withdrawal(1, 101, Decimal::new(100000, 4));
        assert!(engine.execute(withdrawal).is_ok());
        let dispute = Transaction::Dispute(1, 101);
        assert_eq!(
            engine.execute(dispute).err(),
            Some(ExecutionError::IneligibleTransaction)
        );
    }

    #[test]
    fn test_execution_already_disputed_transaction() {
        let mut engine = Engine::new();
        let deposit = Transaction::Deposit(1, 100, Decimal::new(100000, 4));
        assert!(engine.execute(deposit).is_ok());
        let dispute = Transaction::Dispute(1, 100);
        assert!(engine.execute(dispute).is_ok());
        let dispute_again = Transaction::Dispute(1, 100);
        assert_eq!(
            engine.execute(dispute_again).err(),
            Some(ExecutionError::AlreadyDisputedTransaction)
        );
    }
}
