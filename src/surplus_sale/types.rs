use bigdecimal::BigDecimal;
use chrono::{DateTime, Local};
use derive_more::Display;
use getset::Getters;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct Datafile {
    /// The date and time of the auction
    auction_date: DateTime<Local>,
    /// A sorted list of callsigns that have been used in the auction
    callsigns: Vec<Callsign>,
    /// A sorted (by lot number) list of items from the auction
    items: Vec<Item>,
    /// A list of entries for an audit log
    audit_log: Vec<AuditEntry>,
}

impl Datafile {
    #[must_use]
    pub fn new() -> Self {
        Self {
            auction_date: Local::now(),
            callsigns: vec![],
            items: vec![],
            audit_log: vec![AuditEntry::new(AuditItem::Created)],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Item {
    /// The unique lot number for this item
    lot_number: String,
    /// The callsign of the seller
    seller_callsign: Callsign,
    /// The description of this item
    item_description: String,
    /// The reserve price of this item
    reserve_price: Option<BigDecimal>,
    /// Details about the item's sale, if it was successful
    sold_details: Option<SoldDetails>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct SoldDetails {
    /// What price was this sold for under the hammer?
    hammer_price: BigDecimal,
    /// The callsign of the buyer
    buyer_callsign: Callsign,
    /// Has the buyer reconciled against this item?
    buyer_reconciled: bool,
    /// Has the seller reconciled against this item?
    seller_reconciled: bool,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Callsign {
    /// The individual callsign, or if they do not have one allocated, a
    /// callsign-like reference, for example their forename.
    callsign: String,
    /// The individual's name.
    name: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Display, Getters)]
#[display("{moment}: {item}")]
#[getset(get = "pub")]
pub struct AuditEntry {
    /// The moment the audit event happened
    moment: DateTime<Local>,
    /// The item that occurred
    item: AuditItem,
}

impl AuditEntry {
    #[must_use]
    pub fn new(item: AuditItem) -> Self {
        Self {
            moment: Local::now(),
            item,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Display)]
pub enum AuditItem {
    #[display("The auction was created")]
    Created,
    #[display("Lot {lot_number} ({description}) sold to {buyer} for {amount}")]
    LotSold {
        lot_number: String,
        description: String,
        buyer: String,
        amount: BigDecimal,
    },
    #[display("Lot {lot_number} ({description}) did not sell")]
    LotNotSold {
        lot_number: String,
        description: String,
    },
    #[display("Buyer {buyer} has been reconciled against lot {lot_number}")]
    BuyerReconciled { buyer: String, lot_number: String },
    #[display("Seller {seller} has been reconciled against lot {lot_number}")]
    SellerReconciled { seller: String, lot_number: String },
}
