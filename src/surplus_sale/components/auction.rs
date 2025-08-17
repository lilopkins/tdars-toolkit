use std::{rc::Rc, str::FromStr};

use bigdecimal::{BigDecimal, Zero};
use dioxus::prelude::*;
use dioxus_primitives::{label::Label, separator::Separator};

use crate::{
    components::CallsignEntry,
    surplus_sale::{
        types::{Datafile, Item},
        NeedsSaving,
    },
    types::Callsign,
};

#[component]
pub fn Auction() -> Element {
    let mut datafile: Signal<Datafile> = use_context();
    let mut needs_saving: Signal<NeedsSaving> = use_context();
    let mut seller = use_signal(Callsign::default);
    let mut seller_callsign_elem: Signal<Option<Rc<MountedData>>> = use_signal(|| None);
    let lot_number = use_memo(move || {
        if seller.read().callsign().is_empty() {
            String::new()
        } else {
            format!(
                "{}-{}",
                seller().callsign(),
                datafile.read().next_lot_number_for(seller())
            )
        }
    });
    let mut item_description = use_signal(String::new);

    let mut buyer = use_signal(Callsign::default);
    let mut hammer_price = use_signal(BigDecimal::zero);

    let sell_item = move |sold| async move {
        // Set file needs saving
        needs_saving.set(NeedsSaving(true));

        if sold {
            // Save sale
            let mut item = Item::new(lot_number(), seller(), item_description());
            item.sold(hammer_price(), buyer());
            datafile.write().push_item(item);
        } else {
            // Save not sold
            datafile
                .write()
                .push_item(Item::new(lot_number(), seller(), item_description()));
        }

        // Reset sale fields
        seller.set(Callsign::default());
        item_description.set(String::new());
        hammer_price.set(BigDecimal::zero());
        buyer.set(Callsign::default());
        if let Some(seller_callsign_elem) = seller_callsign_elem() {
            _ = seller_callsign_elem.set_focus(true).await;
        }
    };

    rsx! {
        div { display: "flex", flex_direction: "column", gap: "1rem",
            div { display: "flex", flex_direction: "column", gap: ".5rem",
                Label { class: "label", html_for: "lot_number", "Lot number (generated automatically)" }

                input {
                    class: "input",
                    id: "lot_number",
                    placeholder: "M0ABC-1",
                    value: "{lot_number}",
                    readonly: true,
                }
            }

            CallsignEntry {
                suggestion_source: datafile.read().callsigns().clone(),
                value: seller,
                on_mounted_callsign: move |e| seller_callsign_elem.set(e),
                id_prefix: "seller-",
                label_prefix: "Seller's",
            }

            div { display: "flex", flex_direction: "column", gap: ".5rem",
                Label { class: "label", html_for: "description", "Item description" }

                input {
                    class: "input",
                    id: "description",
                    placeholder: "2m dipole",
                    value: "{item_description}",
                    oninput: move |e| item_description.set(e.value()),
                }
            }

            Separator { class: "separator", decorative: false, horizontal: true }

            div { display: "flex", flex_direction: "row", gap: "2rem",
                div {
                    button {
                        class: "button",
                        "data-style": "secondary",
                        onclick: move |_| async move {
                            sell_item(false).await;
                        },
                        "Item not sold"
                    }
                }
                span { "OR" }
                div { display: "flex", flex_direction: "column", gap: ".5rem",
                    CallsignEntry {
                        suggestion_source: datafile.read().callsigns().clone(),
                        value: buyer,
                        id_prefix: "buyer-",
                        label_prefix: "Buyer's",
                    }
                    div {
                        display: "flex",
                        flex_direction: "column",
                        gap: ".5rem",
                        Label { class: "label", html_for: "hammer-price", "Hammer price" }

                        input {
                            class: "input",
                            id: "hammer-price",
                            r#type: "number",
                            step: "0.01",
                            min: "0",
                            placeholder: "123.45",
                            value: "{hammer_price}",
                            oninput: move |e| {
                                if let Ok(p) = BigDecimal::from_str(&e.value()) {
                                    hammer_price.set(p);
                                }
                            },
                            onkeyup: move |e| async move {
                                if e.key() == Key::Enter {
                                    sell_item(true).await;
                                }
                            },
                        }
                    }
                    button {
                        class: "button",
                        "data-style": "primary",
                        onclick: move |_| async move {
                            sell_item(true).await;
                        },
                        "Item sold"
                    }
                }
            }
        }
    }
}
