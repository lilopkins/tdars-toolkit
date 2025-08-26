use bigdecimal::{BigDecimal, Zero};
use chrono::{DateTime, Local};
use derive_more::Display;
use getset::{Getters, MutGetters, Setters};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct Datafile {
    items: Vec<Item>,
    documents: Vec<Document>,
}

impl Datafile {
    /// Create a new datafile
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: vec![],
            documents: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Getters, Setters, Display)]
#[getset(get = "pub")]
#[display("{name} ({price})")]
pub struct Item {
    barcode: String,
    #[getset(set = "pub")]
    name: String,
    #[getset(set = "pub")]
    description: String,
    #[getset(set = "pub")]
    price: BigDecimal,
}

impl Item {
    /// Create a new empty item with just a barcode
    #[must_use]
    pub fn new(barcode: String) -> Self {
        Self {
            barcode,
            name: String::new(),
            description: String::new(),
            price: BigDecimal::zero(),
        }
    }
}

/// Documents represent both receipts and payments
#[derive(Serialize, Deserialize, PartialEq)]
pub struct Document {
    timestamp: DateTime<Local>,
    document_number: String,
    kind: DocumentKind,
    amount: BigDecimal,
}

#[derive(Serialize, Deserialize, PartialEq, Display)]
pub enum DocumentKind {
    #[display("Receipt for {} items", items.len())]
    Receipt { items: Vec<Item> },
    #[display("Payment via {method}")]
    Payment { method: TransactionMethod },
}

#[derive(Serialize, Deserialize, PartialEq, Display)]
pub enum TransactionMethod {
    Cash,
    Card,
    #[display("Bank Transfer")]
    BankTransfer,
    Cheque,
}
