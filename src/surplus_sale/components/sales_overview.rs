use bigdecimal::Zero;
use dioxus::prelude::*;

use crate::surplus_sale::types::Datafile;

#[component]
pub fn SalesOverview() -> Element {
    let datafile: Signal<Datafile> = use_context();
    let sym = use_memo(move || datafile.read().currency().symbol());
    // TODO Allow entries to be revoked so they can be re-entered.

    rsx! {
        table { class: "table",
            thead {
                tr {
                    th { "Lot number" }
                    th { "Item description" }
                    th { "Seller" }
                    th { "Sold for" }
                    th { "Buyer" }
                    th { "S reconciled" }
                    th { "B reconciled" }
                }
            }
            tbody {
                for (callsign , liability) in datafile.read().callsign_liabilities() {
                    if !liability.is_zero() {
                        tr { key: "{callsign}",
                            td {}
                            td {
                                em { "Unpaid amounts" }
                            }
                            td {}
                            td { "{sym} {liability:0.02}" }
                            td { "{callsign}" }
                            td { colspan: 2 }
                        }
                    }
                }
                if datafile.read().items().is_empty() {
                    tr {
                        td { colspan: 7, "No items sold yet..." }
                    }
                }
                for item in datafile.read().items() {
                    tr { key: "{item.lot_number()}",
                        td { "{item.lot_number()}" }
                        td { "{item.description()}" }
                        td { "{item.seller_callsign()}" }
                        if let Some(sold) = item.sold_details() {
                            td { "{sym} {sold.hammer_price():0.02}" }
                            td { "{sold.buyer_callsign()}" }
                            if *sold.seller_reconciled() {
                                td { "✅" }
                            } else {
                                td { "❌" }
                            }
                            if *sold.buyer_reconciled() {
                                td { "✅" }
                            } else {
                                td { "❌" }
                            }
                        } else {
                            td { colspan: 4, "Item not sold." }
                        }
                    }
                }
            }
        }
    }
}
