use rust_decimal::Decimal;
use serde::Deserialize;
use serde::Serialize;


#[allow(non_camel_case_types)]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TransactionType {
    deposit,
    withdrawal,
    dispute,
    resolve,
    chargeback,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    pub client: u16,
    pub tx: u32,
    #[serde(deserialize_with = "csv::invalid_option")]
    pub amount: Option<Decimal>,
}
