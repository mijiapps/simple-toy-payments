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
    use std::collections::HashMap;
    use rust_decimal_macros::dec;
    use crate::data::account::Account;
    use crate::data::transaction::{Transaction, TransactionType};
    use crate::utils::csv_utils::get_test_data;

    #[test]
    fn test_process_deposit() {
        let transaction = Transaction {
            transaction_type: TransactionType::deposit,
            client: 1,
            tx: 1,
            amount: Some(dec!(1.0)),
        };

        let mut account = Account {
            client: 1,
            available: dec!(0),
            held: dec!(0),
            total: dec!(0),
            locked: false,
            transactions: HashMap::new(),
            disputed_transactions: HashMap::new(),
            resolved_transactions: HashMap::new(),
            chargeback_transactions: HashMap::new(),
        };
        account.process_transaction(transaction);
        assert_eq!(account.available, dec!(1.0));
    }

    #[test]
    fn test_process_withdrawal() {
        let transaction = Transaction {
            transaction_type: TransactionType::withdrawal,
            client: 1,
            tx: 1,
            amount: Some(dec!(1.0)),
        };

        let mut account = Account {
            client: 1,
            available: dec!(10),
            held: dec!(0),
            total: dec!(0),
            locked: false,
            transactions: HashMap::new(),
            disputed_transactions: HashMap::new(),
            resolved_transactions: HashMap::new(),
            chargeback_transactions: HashMap::new(),
        };
        account.process_transaction(transaction);
        assert_eq!(account.available, dec!(9.0));
    }

    #[test]
    fn test_process_dispute() {
        let transaction = Transaction {
            transaction_type: TransactionType::deposit,
            client: 1,
            tx: 1,
            amount: Some(dec!(5.0)),
        };

        let disputed_transaction = Transaction {
            transaction_type: TransactionType::dispute,
            client: 1,
            tx: 1,
            amount: Some(dec!(5.0)),
        };

        let mut account = Account {
            client: 1,
            available: dec!(10),
            held: dec!(0),
            total: dec!(10),
            locked: false,
            transactions: HashMap::new(),
            disputed_transactions: HashMap::new(),
            resolved_transactions: HashMap::new(),
            chargeback_transactions: HashMap::new(),
        };
        account.process_transaction(transaction);
        account.process_transaction(disputed_transaction);
        assert_eq!(account.total, dec!(15.0));
        assert_eq!(account.held, dec!(5.0));
        assert_eq!(account.available, dec!(10.0));
    }

    #[test]
    fn test_process_resolve() {
        let transaction = Transaction {
            transaction_type: TransactionType::deposit,
            client: 1,
            tx: 1,
            amount: Some(dec!(5.0)),
        };

        let disputed_transaction = Transaction {
            transaction_type: TransactionType::dispute,
            client: 1,
            tx: 1,
            amount: Some(dec!(5.0)),
        };

        let resolve_transaction = Transaction {
            transaction_type: TransactionType::resolve,
            client: 1,
            tx: 1,
            amount: Some(dec!(5.0)),
        };

        let mut account = Account {
            client: 1,
            available: dec!(10),
            held: dec!(0),
            total: dec!(10),
            locked: false,
            transactions: HashMap::new(),
            disputed_transactions: HashMap::new(),
            resolved_transactions: HashMap::new(),
            chargeback_transactions: HashMap::new(),
        };
        account.process_transaction(transaction);
        account.process_transaction(disputed_transaction);
        account.process_transaction(resolve_transaction);
        assert_eq!(account.total, dec!(15.0));
        assert_eq!(account.held, dec!(0));
        assert_eq!(account.available, dec!(15.0));
    }

    #[test]
    fn test_process_chargeback() {
        let transaction = Transaction {
            transaction_type: TransactionType::deposit,
            client: 1,
            tx: 1,
            amount: Some(dec!(5.0)),
        };

        let disputed_transaction = Transaction {
            transaction_type: TransactionType::dispute,
            client: 1,
            tx: 1,
            amount: Some(dec!(5.0)),
        };

        let chargeback_transaction = Transaction {
            transaction_type: TransactionType::chargeback,
            client: 1,
            tx: 1,
            amount: Some(dec!(5.0)),
        };

        let mut account = Account {
            client: 1,
            available: dec!(10),
            held: dec!(0),
            total: dec!(10),
            locked: false,
            transactions: HashMap::new(),
            disputed_transactions: HashMap::new(),
            resolved_transactions: HashMap::new(),
            chargeback_transactions: HashMap::new(),
        };
        account.process_transaction(transaction);
        account.process_transaction(disputed_transaction);
        account.process_transaction(chargeback_transaction);
        assert_eq!(account.total, dec!(10.0));
        assert_eq!(account.held, dec!(0));
        assert_eq!(account.available, dec!(10.0));
        assert!(account.locked);
    }

    #[test]
    fn test_csv_add_deposits_and_withdrawals() {
        let test_accounts_map = get_test_data("test_data/transactions_comma.csv");
        assert_eq!(test_accounts_map.get(&1).unwrap().available, dec!(6.7599));
    }

    #[test]
    fn test_csv_dispute() {
        let test_accounts_map = get_test_data("test_data/transactions_comma.csv");
        assert_eq!(test_accounts_map.get(&2).unwrap().available, dec!(9.0));
        assert_eq!(test_accounts_map.get(&2).unwrap().held, dec!(6.0));
        assert_eq!(test_accounts_map.get(&2).unwrap().total, dec!(15.0));
    }

    #[test]
    fn test_csv_resolve() {
        let test_accounts_map = get_test_data("test_data/transactions_comma.csv");
        assert_eq!(test_accounts_map.get(&3).unwrap().available, dec!(15.0));
        assert_eq!(test_accounts_map.get(&3).unwrap().held, dec!(0.0));
        assert_eq!(test_accounts_map.get(&3).unwrap().total, dec!(15.0));
    }

    #[test]
    fn test_csv_chargeback() {
        let test_accounts_map = get_test_data("test_data/transactions_comma.csv");
        assert_eq!(test_accounts_map.get(&4).unwrap().available, dec!(9.0));
        assert_eq!(test_accounts_map.get(&4).unwrap().held, dec!(0.0));
        assert_eq!(test_accounts_map.get(&4).unwrap().total, dec!(9.0));
        assert!(test_accounts_map.get(&4).unwrap().locked);
    }
}
