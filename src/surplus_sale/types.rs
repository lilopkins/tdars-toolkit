#![allow(clippy::ref_option)]

use std::{fmt, str::FromStr};

use bigdecimal::{BigDecimal, Zero};
use chrono::{DateTime, Local};
use derive_more::Display;
use dioxus::logger::tracing;
use getset::Getters;
use iso_currency::Currency;
use serde::{Deserialize, Serialize};

use crate::types::Callsign;

#[derive(Serialize, Deserialize, Clone, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct Datafile {
    /// The date and time of the auction
    auction_date: DateTime<Local>,
    /// The club taking as a multiplier (i.e. a 10% taking is stored as
    /// 0.1)
    club_taking: BigDecimal,
    /// The currency this auction was held under
    currency: Currency,
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
        let currency = Currency::GBP;
        #[allow(clippy::unwrap_used, reason = "Default value is validated statically.")]
        let club_taking = BigDecimal::from_str("0.1").unwrap();
        Self {
            auction_date: Local::now(),
            club_taking: club_taking.clone(),
            currency,
            callsigns: vec![],
            items: vec![],
            audit_log: vec![AuditEntry::new(AuditItem::Created {
                currency: currency,
                club_taking_pct: club_taking * 100,
            })],
        }
    }

    /// Return the next lot number for the provided callsign
    pub fn next_lot_number_for(&self, callsign: Callsign) -> i32 {
        let mut next = 1;
        let cs = callsign.callsign();
        loop {
            if self
                .items
                .iter()
                .find(|i| {
                    i.seller_callsign().callsign() == cs
                        && *i.lot_number() == format!("{cs}-{next}")
                })
                .is_none()
            {
                break;
            }
            next += 1;
        }
        next
    }

    /// Set the currency of the auction
    pub fn set_currency(&mut self, currency: Currency) -> &mut Self {
        if currency == self.currency {
            // If there is no change, don't continue
            return self;
        }

        let old_currency = self.currency;
        self.currency = currency;
        self.audit_log
            .push(AuditEntry::new(AuditItem::CurrencyChanged {
                from: old_currency,
                to: currency,
            }));
        self
    }

    /// Set the club taking of the auction
    pub fn set_club_taking(&mut self, club_taking: BigDecimal) -> &mut Self {
        if club_taking == self.club_taking {
            // If there is no change, don't continue
            return self;
        }

        let old_club_taking = self.club_taking.clone();
        self.club_taking = club_taking.clone();
        self.audit_log
            .push(AuditEntry::new(AuditItem::ClubTakingChanged {
                from_pct: old_club_taking * 100,
                to_pct: club_taking * 100,
            }));
        self
    }

    /// Push an item, sold or unsold
    pub fn push_item(&mut self, sale: Item) -> &mut Self {
        let cs = sale.seller_callsign.clone();
        if !self.callsigns.contains(&cs) {
            self.callsigns.push(cs);
        }
        let cs = sale
            .sold_details
            .as_ref()
            .map(|s| s.buyer_callsign())
            .cloned();
        if let Some(cs) = cs {
            if !self.callsigns.contains(&cs) {
                self.callsigns.push(cs);
            }
        }

        if let Some(sold) = sale.sold_details() {
            self.audit_log.push(AuditEntry::new(AuditItem::LotSold {
                lot_number: sale.lot_number().clone(),
                description: sale.description().clone(),
                seller: sale.seller_callsign().clone(),
                buyer: sold.buyer_callsign().clone(),
                currency: *self.currency(),
                amount: sold.hammer_price().clone(),
            }));
        } else {
            self.audit_log.push(AuditEntry::new(AuditItem::LotNotSold {
                lot_number: sale.lot_number().clone(),
                description: sale.description().clone(),
            }));
        }

        self.items.push(sale);
        self
    }

    /// Reconcile the callsign by the amount. Returns the amount remaining, i.e. change.
    pub fn reconcile(&mut self, callsign: Callsign, mut amount: BigDecimal, all_funds_to_club: bool) -> BigDecimal {
        self.audit_log.push(AuditEntry::new(AuditItem::Reconciled {
            callsign: callsign.clone(),
            amount: amount.clone(),
            currency: *self.currency(),
        }));
        let ct = self.club_taking().clone();
        // Sold items first
        self.items.iter_mut().for_each(|i| {
            if *i.seller_callsign() == callsign {
                // Item sold by CS
                if let Some(sold) = &mut i.sold_details {
                    let amt_less_club: BigDecimal = sold.hammer_price() * (1 - ct.clone());
                    amount += amt_less_club.clone();
                    sold.seller_reconciled = amt_less_club;
                    sold.seller_all_funds_to_club = all_funds_to_club;
                }
            }
        });

        // Then bought items
        self.items.iter_mut().for_each(|i| {
            if i.sold_details()
                .as_ref()
                .is_some_and(|s| *s.buyer_callsign() == callsign)
            {
                // Item bought by CS
                if let Some(sold) = &mut i.sold_details {
                    let amt = sold.hammer_price().clone().min(amount.clone());
                    amount -= amt.clone();
                    sold.buyer_reconciled = amt;
                }
            }
        });

        amount
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct Item {
    /// The unique lot number for this item
    lot_number: String,
    /// The callsign of the seller
    seller_callsign: Callsign,
    /// The description of this item
    description: String,
    /// Details about the item's sale, if it was successful
    sold_details: Option<SoldDetails>,
}

impl Item {
    /// Create a new item
    pub fn new(lot_number: String, seller_callsign: Callsign, description: String) -> Self {
        Self {
            lot_number,
            seller_callsign,
            description,
            sold_details: None,
        }
    }

    /// Mark the item as sold
    pub fn sold(&mut self, hammer_price: BigDecimal, buyer_callsign: Callsign) -> &mut Self {
        self.sold_details = Some(SoldDetails {
            hammer_price,
            buyer_callsign,
            buyer_reconciled: BigDecimal::zero(),
            seller_reconciled: BigDecimal::zero(),
            seller_all_funds_to_club: false,
        });
        self
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct SoldDetails {
    /// What price was this sold for under the hammer?
    hammer_price: BigDecimal,
    /// The callsign of the buyer
    buyer_callsign: Callsign,
    /// How much has the buyer reconciled against this item?
    buyer_reconciled: BigDecimal,
    /// How much has the seller reconciled against this item?
    seller_reconciled: BigDecimal,
    /// Indicates that the seller opted for all revenue to go to the
    /// club
    seller_all_funds_to_club: bool,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct AuditEntry {
    /// The moment the audit event happened
    moment: DateTime<Local>,
    /// The item that occurred
    item: AuditItem,
}

impl fmt::Display for AuditEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let moment = self.moment.format("%F %T%.3f %Z");
        let item = &self.item;
        write!(f, "{moment}: {item}")
    }
}

impl AuditEntry {
    #[must_use]
    pub fn new(item: AuditItem) -> Self {
        tracing::info!("New audit event: {item}");
        Self {
            moment: Local::now(),
            item,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Display)]
pub enum AuditItem {
    #[display(
        "The auction was created with currency {currency} and club taking {club_taking_pct}%"
    )]
    Created {
        currency: Currency,
        club_taking_pct: BigDecimal,
    },
    #[display("The system currency has changed from {from} to {to}")]
    CurrencyChanged { from: Currency, to: Currency },
    #[display("The club taking has changed from {from_pct}% to {to_pct}%")]
    ClubTakingChanged {
        from_pct: BigDecimal,
        to_pct: BigDecimal,
    },
    #[display(
        "Lot {lot_number} ({description}) sold by {seller} to {buyer} for {amount} {currency}"
    )]
    LotSold {
        lot_number: String,
        description: String,
        seller: Callsign,
        buyer: Callsign,
        currency: Currency,
        amount: BigDecimal,
    },
    #[display("Lot {lot_number} ({description}) did not sell")]
    LotNotSold {
        lot_number: String,
        description: String,
    },
    #[display("{callsign} has reconciled {amount} {currency}")]
    Reconciled {
        callsign: Callsign,
        amount: BigDecimal,
        currency: Currency,
    },
}
