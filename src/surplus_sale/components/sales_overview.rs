use bigdecimal::Zero;
use dioxus::prelude::*;

use crate::surplus_sale::types::Datafile;

#[component]
pub fn SalesOverview() -> Element {
    let mut datafile: Signal<Datafile> = use_context();
    let sym = use_memo(move || datafile.read().currency().symbol());

    let mut delete_item = move |lot_nmr| {
        datafile.write().delete_item(lot_nmr);
    };

    rsx! {
        table { class: "table",
            thead {
                tr {
                    th { "Lot number" }
                    th { "Item description" }
                    th { "Seller" }
                    th { "Sold for" }
                    th { "Buyer" }
                    th { "Reconciled (S/B)" }
                    th {}
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
                            td {
                                if *sold.seller_reconciled() {
                                    "✅"
                                } else {
                                    "❌"
                                }
                                " / "
                                if *sold.buyer_reconciled() {
                                    "✅"
                                } else {
                                    "❌"
                                }
                            }
                        } else {
                            td { colspan: 3, "Item not sold." }
                        }

                        if item.sold_details().as_ref().is_some_and(|sold| *sold.buyer_reconciled() || *sold.seller_reconciled()) {
                            td {}
                        } else {
                            td {
                                button {
                                    class: "button",
                                    "data-style": "destructive",
                                    onclick: {
                                        let lot_nmr = item.lot_number().clone();
                                        move |_| delete_item(lot_nmr.clone())
                                    },
                                    "Revoke"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
