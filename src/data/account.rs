use std::collections::HashMap;

use rust_decimal::Decimal;
use serde::Deserialize;
use serde::Serialize;

use crate::data::transaction::{Transaction, TransactionType};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Account {
    pub client: u16,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,
    #[serde(skip_serializing)]
    pub transactions: HashMap<u32, Transaction>,
    #[serde(skip_serializing)]
    pub disputed_transactions: HashMap<u32, Transaction>,
    #[serde(skip_serializing)]
    pub resolved_transactions: HashMap<u32, Transaction>,
    #[serde(skip_serializing)]
    pub chargeback_transactions: HashMap<u32, Transaction>,
}

impl Account {

    pub fn process_transaction(&mut self, transaction: Transaction) {
        let tx = transaction.tx.clone();
        let transaction_amount = transaction.amount.unwrap_or_default();
        match transaction.transaction_type {
            TransactionType::deposit => {
                self.available += transaction_amount;
                self.total += transaction_amount;
                self.transactions.insert(tx, transaction);
            },
            TransactionType::withdrawal => {
                if self.available < transaction_amount {
                    return;
                }
                self.available -= transaction_amount;
                self.total -= transaction_amount;
                self.transactions.insert(tx, transaction);
            },
            TransactionType::dispute => {
                if !self.transaction_exists(&transaction.tx) {
                    return;
                }
                let disputed_amt = self.transactions.get(&transaction.tx)
                    .and_then(|t| t.amount)
                    .unwrap_or_default();

                self.available -= disputed_amt;
                self.held += disputed_amt;
                self.disputed_transactions.insert(tx, transaction);
            },
            TransactionType::resolve => {
                if !self.transaction_exists(&transaction.tx) {
                    return;
                }
                let resolved_amt = self.transactions.get(&transaction.tx)
                    .and_then(|t| t.amount)
                    .unwrap_or_default();

                self.held -= resolved_amt;
                self.available += resolved_amt;
                self.resolved_transactions.insert(tx, transaction);
            },
            TransactionType::chargeback => {
                if !self.transaction_exists(&transaction.tx) {
                    return;
                }
                let chargeback_amt = self.transactions.get(&transaction.tx)
                    .and_then(|t| t.amount)
                    .unwrap_or_default();

                self.held -= chargeback_amt;
                self.total -= chargeback_amt;
                self.locked = true;
                self.chargeback_transactions.insert(tx, transaction);
            }
        }
    }

    fn transaction_exists(&self, tx: &u32) -> bool {
        self.transactions.contains_key(tx)
    }
}

#[cfg(test)]
mod tests {
    use rust_decimal_macros::dec;
    use crate::utils::csv_utils::get_test_data;

    #[test]
    fn test_add_deposits_and_withdrawals() {
        let test_accounts_map = get_test_data();
        assert_eq!(test_accounts_map.get(&1).unwrap().available, dec!(6.7599));
    }

    #[test]
    fn test_dispute() {
        let test_accounts_map = get_test_data();
        assert_eq!(test_accounts_map.get(&2).unwrap().available, dec!(9.0));
        assert_eq!(test_accounts_map.get(&2).unwrap().held, dec!(6.0));
        assert_eq!(test_accounts_map.get(&2).unwrap().total, dec!(15.0));
    }

    #[test]
    fn test_resolve() {
        let test_accounts_map = get_test_data();
        assert_eq!(test_accounts_map.get(&3).unwrap().available, dec!(15.0));
        assert_eq!(test_accounts_map.get(&3).unwrap().held, dec!(0.0));
        assert_eq!(test_accounts_map.get(&3).unwrap().total, dec!(15.0));
    }

    #[test]
    fn test_chargeback() {
        let test_accounts_map = get_test_data();
        assert_eq!(test_accounts_map.get(&4).unwrap().available, dec!(9.0));
        assert_eq!(test_accounts_map.get(&4).unwrap().held, dec!(0.0));
        assert_eq!(test_accounts_map.get(&4).unwrap().total, dec!(9.0));
        assert!(test_accounts_map.get(&4).unwrap().locked);
    }
}
