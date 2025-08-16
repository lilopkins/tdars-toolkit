use dioxus::prelude::*;

use crate::surplus_sale::types::Datafile;

#[component]
pub fn SalesOverview() -> Element {
    let datafile: Signal<Datafile> = use_context();

    rsx! {
        table { class: "table",
            thead {
                tr {
                    th { "Lot number" }
                    th { "Item description" }
                    th { "Seller" }
                    th { "Sold for" }
                    th { "Buyer" }
                    th { "Seller reconciled" }
                    th { "Buyer reconciled" }
                }
            }
            tbody {
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
                            td { "{sold.hammer_price():0.02}" }
                            td { "{sold.buyer_callsign()}" }
                            if *sold.seller_reconciled() == (sold.hammer_price() * (1 - datafile().club_taking())) {
                                td { "✅" }
                            } else {
                                td { "{sold.seller_reconciled():0.02}" }
                            }
                            if sold.buyer_reconciled() == sold.hammer_price() {
                                td { "✅" }
                            } else {
                                td { "{sold.buyer_reconciled():0.02}" }
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
