use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::io;
use csv::Reader;
use crate::data::account::Account;
use crate::data::transaction::Transaction;


pub fn create_reader(filepath: OsString) -> Result<Box<Reader<File>>, Box<dyn Error>> {
    let reader = csv::ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .from_path(filepath)?;
    Ok(Box::new(reader))
}

pub fn parse_csv_data(reader: &mut Reader<File>) -> Result<Box<HashMap<u16, Account>>, Box<dyn Error>> {
    let mut accounts_map: HashMap<u16, Account> = HashMap::new();

    for result in reader.deserialize() {
        let transaction: Transaction = result?;

        match accounts_map.entry(transaction.client.clone()) {
            Entry::Vacant(entry) => {
                let mut account = Account {
                    client: entry.key().clone(),
                    available: Default::default(),
                    held: Default::default(),
                    total: Default::default(),
                    locked: false,
                    transactions: Default::default(),
                    disputed_transactions: Default::default(),
                    resolved_transactions: Default::default(),
                    chargeback_transactions: Default::default(),
                };
                account.process_transaction(transaction);
                entry.insert(account);
            }
            Entry::Occupied(mut entry) => {
                entry.get_mut().process_transaction(transaction);
            }
        }
    }
    Ok(Box::new(accounts_map))
}

pub fn write_csv_data(accounts: HashMap<u16, Account>) -> Result<(), Box<dyn Error>> {
    let mut writer = csv::Writer::from_writer(io::stdout());
    for (_, account) in accounts {
        writer.serialize(account)?;
    }
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
pub fn get_test_data() -> Box<HashMap<u16, Account>> {
    let mut reader = create_reader("test_data/transactions_comma.csv".into()).expect("Could not open file");
    return parse_csv_data(&mut reader).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csv_data_not_empty() {
        let test_accounts_map = get_test_data();
        assert_ne!(test_accounts_map.len(), 0);
    }

    #[test]
    fn test_parse_csv_data_contents() {
        let test_accounts_map = get_test_data();
        assert_eq!(test_accounts_map.get(&1).unwrap().transactions.len(), 4);
        assert_eq!(test_accounts_map.get(&2).unwrap().transactions.len(), 2);
        assert_eq!(test_accounts_map.get(&2).unwrap().disputed_transactions.len(), 1);
    }
}
