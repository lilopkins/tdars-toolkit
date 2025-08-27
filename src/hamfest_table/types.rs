use bigdecimal::{BigDecimal, Zero};
use chrono::{DateTime, Local};
use derive_more::Display;
use getset::{Getters, MutGetters, Setters};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, PartialEq, Getters, MutGetters)]
#[getset(get = "pub", get_mut = "pub")]
pub struct Datafile {
    items: Vec<Item>,
    receipts: Vec<Receipt>,
}

impl Datafile {
    /// Create a new datafile
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: vec![],
            receipts: vec![],
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Getters, Setters, Display, Default, Clone)]
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
#[derive(Serialize, Deserialize, PartialEq, Getters, MutGetters, Clone)]
#[getset(get = "pub")]
pub struct Receipt {
    timestamp: DateTime<Local>,
    receipt_number: Uuid,
    #[getset(get_mut = "pub")]
    lines: Vec<ReceiptLine>,
}

impl Receipt {
    /// Create a new receipt
    #[must_use]
    pub fn new() -> Self {
        Self {
            timestamp: Local::now(),
            receipt_number: Uuid::new_v4(),
            lines: vec![],
        }
    }

    /// Calculate the total of this receipt
    #[must_use]
    pub fn total(&self) -> BigDecimal {
        let mut total = BigDecimal::zero();
        for line in self.lines() {
            match line {
                ReceiptLine::Item { item } => total += item.price(),
                ReceiptLine::Payment { amount, .. } => total -= amount,
                ReceiptLine::Change { amount, .. } => total += amount,
            }
        }
        total
    }
}

#[derive(Serialize, Deserialize, PartialEq, Display, Clone)]
pub enum ReceiptLine {
    Item {
        item: Item,
    },
    #[display("Payment for {amount:0.02} via {method}")]
    Payment {
        method: TransactionMethod,
        amount: BigDecimal,
    },
    #[display("Change for {amount:0.02} via {method}")]
    Change {
        method: TransactionMethod,
        amount: BigDecimal,
    },
}

#[derive(Serialize, Deserialize, PartialEq, Display, Clone, Copy)]
pub enum TransactionMethod {
    Cash,
    Card,
    #[display("Bank Transfer")]
    BankTransfer,
    Cheque,
}
