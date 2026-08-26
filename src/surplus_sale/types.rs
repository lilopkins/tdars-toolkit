#![allow(clippy::ref_option)]

use std::collections::HashMap;
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
    /// A map of callsigns that still owe amounts
    #[serde(default, with = "callsign_liabilities_serde")]
    callsign_liabilities: HashMap<Callsign, CallsignLiability>,
    /// A list of dontations to the club, callsign and amount
    club_donations: Vec<(Callsign, BigDecimal)>,
    /// A list of entries for an audit log
    audit_log: Vec<AuditEntry>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Getters, Default)]
#[getset(get = "pub")]
pub struct CallsignLiability {
    /// Outstanding amounts not tied to a specific item
    #[serde(default)]
    amount: BigDecimal,
    /// Payment history and remaining balances for bought items
    #[serde(default)]
    item_payments: Vec<BuyerLiabilityItem>,
}

impl CallsignLiability {
    #[must_use]
    pub fn total(&self) -> BigDecimal {
        self.item_payments
            .iter()
            .fold(self.amount.clone(), |acc, item| acc + item.remaining())
    }

    #[must_use]
    pub fn item_payment(&self, lot_number: &str) -> Option<&BuyerLiabilityItem> {
        self.item_payments
            .iter()
            .find(|item| item.lot_number() == lot_number)
    }

    fn upsert_item_payment(
        &mut self,
        lot_number: String,
        description: String,
        remaining: BigDecimal,
    ) -> &mut BuyerLiabilityItem {
        if let Some(idx) = self
            .item_payments
            .iter()
            .position(|item| item.lot_number() == &lot_number)
        {
            return &mut self.item_payments[idx];
        }

        self.item_payments.push(BuyerLiabilityItem {
            lot_number,
            description,
            remaining,
            payments: vec![],
        });
        #[allow(clippy::unwrap_used, reason = "entry was just inserted")]
        self.item_payments.last_mut().unwrap()
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct BuyerLiabilityItem {
    /// The lot number this liability applies to
    lot_number: String,
    /// The item description at the point of reconciliation
    description: String,
    /// The amount still outstanding for this item
    #[serde(default)]
    remaining: BigDecimal,
    /// Payments made against this item
    #[serde(default)]
    payments: Vec<BuyerLiabilityPayment>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct BuyerLiabilityPayment {
    /// The amount paid in this transaction
    amount: BigDecimal,
    /// How the amount was reconciled
    method: ReconcileMethod,
}

mod callsign_liabilities_serde {
    use std::collections::HashMap;

    use bigdecimal::BigDecimal;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::{BuyerLiabilityItem, CallsignLiability};
    use crate::types::Callsign;

    #[derive(Serialize, Deserialize)]
    struct LiabilityEntry {
        callsign: Callsign,
        amount: BigDecimal,
        #[serde(default)]
        item_payments: Vec<BuyerLiabilityItem>,
    }

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum LiabilityEncoding {
        EntryList(Vec<LiabilityEntry>),
        StringMap(HashMap<String, BigDecimal>),
    }

    pub fn serialize<S>(
        liabilities: &HashMap<Callsign, CallsignLiability>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let entries = liabilities
            .iter()
            .map(|(callsign, liability)| LiabilityEntry {
                callsign: callsign.clone(),
                amount: liability.amount().clone(),
                item_payments: liability.item_payments().clone(),
            })
            .collect::<Vec<_>>();
        entries.serialize(serializer)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<HashMap<Callsign, CallsignLiability>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encoding = LiabilityEncoding::deserialize(deserializer)?;

        match encoding {
            LiabilityEncoding::EntryList(entries) => Ok(entries
                .into_iter()
                .map(|entry| {
                    (
                        entry.callsign,
                        CallsignLiability {
                            amount: entry.amount,
                            item_payments: entry.item_payments,
                        },
                    )
                })
                .collect::<HashMap<_, _>>()),
            LiabilityEncoding::StringMap(entries) => entries
                .into_iter()
                .map(|(key, amount)| {
                    parse_key(&key).map(|callsign| {
                        (
                            callsign,
                            CallsignLiability {
                                amount,
                                item_payments: vec![],
                            },
                        )
                    })
                })
                .collect::<Result<HashMap<_, _>, _>>()
                .map_err(serde::de::Error::custom),
        }
    }

    fn parse_key(key: &str) -> Result<Callsign, String> {
        if let Ok(callsign) = serde_json::from_str::<Callsign>(key) {
            return Ok(callsign);
        }

        if key.trim().is_empty() {
            return Err("callsign liability key cannot be empty".to_string());
        }

        Ok(Callsign::default().with_callsign(key.to_string()))
    }
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
            callsign_liabilities: HashMap::new(),
            club_donations: vec![],
            audit_log: vec![AuditEntry::new(AuditItem::Created {
                currency,
                club_taking_pct: club_taking * 100,
            })],
        }
    }

    /// Return the next lot number for the provided callsign
    pub fn next_lot_number_for(&self, callsign: &Callsign) -> i32 {
        let mut next = 1;
        let cs = callsign.callsign();
        loop {
            if !self.items.iter().any(|i| {
                i.seller_callsign().callsign() == cs && *i.lot_number() == format!("{cs}-{next}")
            }) {
                break;
            }
            next += 1;
        }
        next
    }

    /// Delete an item if it is not at all reconciled.
    pub fn delete_item(&mut self, lot_number: String) {
        let has_recorded_payments = self.callsign_liabilities.values().any(|liability| {
            liability
                .item_payments()
                .iter()
                .any(|item| item.lot_number() == &lot_number && !item.payments().is_empty())
        });
        self.items.retain(|i| {
            (*i.lot_number() != lot_number)
                || i.sold_details().as_ref().is_some_and(|s| {
                    s.buyer_reconciled().is_some() || s.seller_reconciled().is_some()
                })
                || has_recorded_payments
        });
        self.audit_log
            .push(AuditEntry::new(AuditItem::RevokeItem { lot_number }));
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
            .map(SoldDetails::buyer_callsign)
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
    ///
    /// If the club pays out, `reconcile_amount` should be negative. Inverseley if the
    /// club takes money, `reconcile_amount` should be positive.
    #[allow(clippy::too_many_lines, reason = "reconciliation is kept in one place")]
    pub fn reconcile(
        &mut self,
        callsign: &Callsign,
        mut reconcile_amount: BigDecimal,
        reconcile_method: ReconcileMethod,
    ) -> BigDecimal {
        self.audit_log.push(AuditEntry::new(AuditItem::Reconciled {
            callsign: callsign.clone(),
            amount: reconcile_amount.clone(),
            currency: *self.currency(),
            method: reconcile_method,
        }));
        let ct = self.club_taking().clone();
        let curr = *self.currency();
        // Sold items first
        let mut audit_items = self
            .items
            .iter_mut()
            .filter(|i| i.seller_callsign() == callsign)
            .filter_map(|i| {
                // Item sold by CS
                if let Some(sold) = &mut i.sold_details {
                    if sold.seller_reconciled.is_some() {
                        return None;
                    }
                    let hammer_less_club: BigDecimal = sold.hammer_price() * (1 - ct.clone());
                    let amt = hammer_less_club;
                    reconcile_amount += amt.clone();
                    sold.seller_reconciled = Some(reconcile_method);
                    if reconcile_method == ReconcileMethod::Donation {
                        return Some(AuditEntry::new(AuditItem::DonationToClub {
                            callsign: callsign.clone(),
                            amount: amt.clone(),
                            currency: curr,
                        }));
                    }
                }
                None
            })
            .collect::<Vec<_>>();

        self.audit_log.append(&mut audit_items);

        let mut liability = self
            .callsign_liabilities
            .remove(callsign)
            .unwrap_or_default();

        // Legacy/unmatched liabilities at the highest point
        if reconcile_amount > BigDecimal::zero() {
            let dues_paid = liability.amount().clone().min(reconcile_amount.clone());
            liability.amount -= dues_paid.clone();
            reconcile_amount -= dues_paid;
        }

        // Then bought items
        self.items
            .iter_mut()
            .filter(|i| {
                i.sold_details()
                    .as_ref()
                    .is_some_and(|s| s.buyer_callsign() == callsign)
            })
            .for_each(|i| {
                let lot_number = i.lot_number().clone();
                let description = i.description().clone();

                // Item bought by CS
                if let Some(sold) = &mut i.sold_details {
                    if sold.buyer_reconciled.is_some()
                        && liability.item_payment(&lot_number).is_none()
                    {
                        return;
                    }

                    if reconcile_amount <= BigDecimal::zero() {
                        return;
                    }

                    let entry = liability.upsert_item_payment(
                        lot_number,
                        description,
                        sold.hammer_price().clone(),
                    );
                    let paid = entry.remaining().clone().min(reconcile_amount.clone());
                    if paid <= BigDecimal::zero() {
                        return;
                    }

                    entry.payments.push(BuyerLiabilityPayment {
                        amount: paid.clone(),
                        method: reconcile_method,
                    });
                    entry.remaining -= paid.clone();
                    reconcile_amount -= paid;

                    if entry.remaining().is_zero() {
                        sold.buyer_reconciled = Some(reconcile_method);
                    }
                }
            });

        let still_owed = self
            .items
            .iter()
            .filter(|i| {
                i.sold_details().as_ref().is_some_and(|s| {
                    s.buyer_callsign() == callsign
                        && s.buyer_reconciled().is_none()
                        && liability.item_payment(i.lot_number()).is_none()
                })
            })
            .fold(liability.total(), |acc, i| {
                acc + i
                    .sold_details()
                    .as_ref()
                    .map_or_else(BigDecimal::zero, |sold| sold.hammer_price().clone())
            });

        if still_owed.is_zero() {
            self.audit_log
                .push(AuditEntry::new(AuditItem::ReconciledFully {
                    callsign: callsign.clone(),
                }));
            if reconcile_amount > BigDecimal::zero()
                && reconcile_method != ReconcileMethod::Donation
            {
                // Change returned
                self.audit_log.push(AuditEntry::new(AuditItem::ChangeGiven {
                    callsign: callsign.clone(),
                    amount: reconcile_amount.clone(),
                    currency: *self.currency(),
                }));
            }
        }

        let change = reconcile_amount.max(BigDecimal::zero());
        if change > BigDecimal::zero() && reconcile_method == ReconcileMethod::Donation {
            // Donate change to club
            self.audit_log
                .push(AuditEntry::new(AuditItem::DonationToClub {
                    callsign: callsign.clone(),
                    amount: change.clone(),
                    currency: *self.currency(),
                }));
            self.club_donations.push((callsign.clone(), change.clone()));
            if !liability.amount().is_zero() || !liability.item_payments().is_empty() {
                self.callsign_liabilities
                    .insert(callsign.clone(), liability);
            }
            BigDecimal::zero()
        } else {
            if !liability.amount().is_zero() || !liability.item_payments().is_empty() {
                self.callsign_liabilities
                    .insert(callsign.clone(), liability);
            }
            change
        }
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
            buyer_reconciled: None,
            seller_reconciled: None,
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
    /// Has the buyer reconciled against this item?
    buyer_reconciled: Option<ReconcileMethod>,
    /// Has the seller reconciled against this item?
    seller_reconciled: Option<ReconcileMethod>,
}

/// How was the amount reconciled?
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Display)]
pub enum ReconcileMethod {
    /// The buyer/seller (was) paid with cash
    #[display("Cash")]
    Cash,
    /// The seller donated the funds to the club (seller only)
    #[display("Donation")]
    Donation,
    /// The buyer/seller (was) paid by bank transfer
    #[display("Bank Xfr ({})", if *seen { "seen" } else { "unseen" })]
    BankTransfer {
        /// Was evidence of the bank transfer seen?
        seen: bool,
    },
    /// The buyer/seller has agreed to pay at a later date
    #[display("Postponed")]
    Postpone,
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
    #[display("{callsign} has reconciled {amount} {currency} via {method}")]
    Reconciled {
        callsign: Callsign,
        amount: BigDecimal,
        currency: Currency,
        method: ReconcileMethod,
    },
    #[display("{callsign} has donated {amount} {currency} to the club")]
    DonationToClub {
        callsign: Callsign,
        amount: BigDecimal,
        currency: Currency,
    },
    #[display("{callsign} has reconciled fully")]
    ReconciledFully { callsign: Callsign },
    #[display("{callsign} has been given change: {amount} {currency}")]
    ChangeGiven {
        callsign: Callsign,
        amount: BigDecimal,
        currency: Currency,
    },
    #[display("The lot {lot_number} has been revoked.")]
    RevokeItem { lot_number: String },
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use bigdecimal::BigDecimal;
    use serde_json::json;

    use super::{CallsignLiability, Datafile, Item, ReconcileMethod};
    use crate::types::Callsign;

    #[test]
    fn liabilities_roundtrip_via_entry_list() {
        let mut datafile = Datafile::new();
        let mut callsign = Callsign::default();
        callsign
            .set_callsign("M0AAA".to_string())
            .set_name("Alice".to_string());
        #[allow(clippy::unwrap_used, reason = "test data is valid")]
        let amount = BigDecimal::from_str("12.50").unwrap();
        datafile.callsign_liabilities.insert(
            callsign.clone(),
            CallsignLiability {
                amount: amount.clone(),
                item_payments: vec![],
            },
        );

        #[allow(clippy::unwrap_used, reason = "test serialization should succeed")]
        let encoded = serde_json::to_value(&datafile).unwrap();
        assert!(encoded["callsign_liabilities"].is_array());

        #[allow(clippy::unwrap_used, reason = "test roundtrip should succeed")]
        let decoded: Datafile = serde_json::from_value(encoded).unwrap();
        let liability = decoded
            .callsign_liabilities()
            .get(&callsign)
            .expect("liability should roundtrip");
        assert_eq!(liability.amount(), &amount);
    }

    #[test]
    fn liabilities_deserialise_legacy_string_map() {
        let encoded = json!({
            "auction_date": "2026-01-01T00:00:00+00:00",
            "club_taking": "0.1",
            "currency": "GBP",
            "callsigns": [],
            "items": [],
            "callsign_liabilities": {
                "M0BBB": "3.75"
            },
            "club_donations": [],
            "audit_log": []
        });

        #[allow(clippy::unwrap_used, reason = "legacy fixture should deserialize")]
        let decoded: Datafile = serde_json::from_value(encoded).unwrap();
        let callsign = Callsign::default().with_callsign("M0BBB".to_string());
        #[allow(clippy::unwrap_used, reason = "test data is valid")]
        let expected = BigDecimal::from_str("3.75").unwrap();
        let liability = decoded
            .callsign_liabilities()
            .get(&callsign)
            .expect("legacy liability should deserialize");
        assert_eq!(liability.amount(), &expected);
        assert!(liability.item_payments().is_empty());
    }

    #[test]
    fn split_payment_liability_can_be_saved() {
        let mut datafile = Datafile::new();

        let mut seller = Callsign::default();
        seller
            .set_callsign("M0SELL".to_string())
            .set_name("Seller".to_string());
        let mut buyer = Callsign::default();
        buyer
            .set_callsign("M0BUY".to_string())
            .set_name("Buyer".to_string());

        #[allow(clippy::unwrap_used, reason = "test data is valid")]
        let mut item = Item::new("M0SELL-1".to_string(), seller, "Rig".to_string());
        item.sold(BigDecimal::from_str("10.00").unwrap(), buyer.clone());
        datafile.push_item(item);
        datafile.reconcile(
            &buyer,
            BigDecimal::from_str("5.00").unwrap(),
            ReconcileMethod::Cash,
        );

        let liability = datafile
            .callsign_liabilities()
            .get(&buyer)
            .expect("split payment liability should exist");
        #[allow(clippy::unwrap_used, reason = "test data is valid")]
        let expected = BigDecimal::from_str("5.00").unwrap();
        assert_eq!(liability.total(), expected);

        #[allow(
            clippy::unwrap_used,
            reason = "split payment datafile should serialize"
        )]
        let encoded = serde_json::to_vec(&datafile).unwrap();
        assert!(!encoded.is_empty());
    }

    #[test]
    fn partial_buyer_payment_stays_unreconciled_until_fully_paid() {
        let mut datafile = Datafile::new();

        let mut seller = Callsign::default();
        seller
            .set_callsign("M0SELL".to_string())
            .set_name("Seller".to_string());
        let mut buyer = Callsign::default();
        buyer
            .set_callsign("M0BUY".to_string())
            .set_name("Buyer".to_string());

        #[allow(clippy::unwrap_used, reason = "test data is valid")]
        let mut item = Item::new("M0SELL-1".to_string(), seller, "Rig".to_string());
        item.sold(BigDecimal::from_str("10.00").unwrap(), buyer.clone());
        datafile.push_item(item);
        datafile.reconcile(
            &buyer,
            BigDecimal::from_str("5.00").unwrap(),
            ReconcileMethod::Cash,
        );

        let sold = datafile.items()[0]
            .sold_details()
            .as_ref()
            .expect("sold details should exist");
        assert!(sold.buyer_reconciled().is_none());

        let liability = datafile
            .callsign_liabilities()
            .get(&buyer)
            .expect("split payment liability should exist");
        assert_eq!(liability.item_payments().len(), 1);
        #[allow(clippy::unwrap_used, reason = "test data is valid")]
        let remaining = BigDecimal::from_str("5.00").unwrap();
        assert_eq!(liability.item_payments()[0].remaining(), &remaining);

        datafile.reconcile(
            &buyer,
            BigDecimal::from_str("5.00").unwrap(),
            ReconcileMethod::BankTransfer { seen: true },
        );

        let sold = datafile.items()[0]
            .sold_details()
            .as_ref()
            .expect("sold details should exist");
        assert!(sold.buyer_reconciled() == &Some(ReconcileMethod::BankTransfer { seen: true }));
        let liability = datafile
            .callsign_liabilities()
            .get(&buyer)
            .expect("split payment liability should exist");
        assert_eq!(liability.item_payments()[0].payments().len(), 2);
    }
}
