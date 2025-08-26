use std::{rc::Rc, str::FromStr};

use bigdecimal::{BigDecimal, Zero};
use dioxus::{logger::tracing, prelude::*};
use dioxus_primitives::label::Label;

use crate::hamfest_table::types::{Datafile, Item};

#[component]
pub fn LoadedFile(datafile: MappedMutSignal<Datafile, Signal<Option<Datafile>>>) -> Element {
    let mut barcode = use_signal(String::new);
    let mut barcode_elem: Signal<Option<Rc<MountedData>>> = use_signal(|| None);

    let mut item_name = use_signal(String::new);
    let mut item_description = use_signal(String::new);
    let mut item_price = use_signal(BigDecimal::zero);
    use_effect(move || {
        // Reload item details from barcode
        let barcode = barcode();
        if barcode.is_empty() {
            item_name.set(String::new());
            item_description.set(String::new());
            item_price.set(BigDecimal::zero());
        } else {
            // Try to fetch item from barcode
            if let Some(item) = datafile
                .peek()
                .items()
                .iter()
                .find(|i| *i.barcode() == barcode)
            {
                tracing::info!("Barcode changed and item found! {item}");
                item_name.set(item.name().clone());
                item_description.set(item.description().clone());
                item_price.set(item.price().clone());
            } else {
                tracing::info!("Barcode changed to nonexistant item.");
                item_name.set(String::new());
                item_description.set(String::new());
                item_price.set(BigDecimal::zero());
            }
        }
    });
    use_effect(move || {
        // Update item when details changed/created
        let barcode = barcode.peek();
        let item_name = item_name();
        let item_description = item_description();
        let item_price = item_price();
        if barcode.is_empty() || item_name.is_empty() || item_price == BigDecimal::zero() {
            // Ignore
            return;
        }

        // Create new item if it doesn't exist yet
        if !datafile
            .peek()
            .items()
            .iter()
            .any(|i| *i.barcode() == *barcode)
        {
            tracing::info!("New item created!");
            datafile
                .write()
                .items_mut()
                .push(Item::new(barcode.clone()));
        }
        // Update item params
        if let Some(item) = datafile
            .write()
            .items_mut()
            .iter_mut()
            .find(|i| *i.barcode() == *barcode)
        {
            tracing::info!("Item updated");
            item.set_name(item_name)
                .set_description(item_description)
                .set_price(item_price);
        }
    });

    rsx! {
        div { display: "flex", gap: "1rem",
            div { display: "flex", flex_direction: "column", gap: "1rem",
                // Search input
                div { display: "flex", gap: ".5rem",
                    div {
                        display: "flex",
                        flex_direction: "column",
                        gap: ".5rem",
                        Label { class: "label", html_for: "barcode", "Item Barcode" }

                        input {
                            class: "input",
                            id: "barcode",
                            value: "{barcode}",
                            onmounted: move |e| barcode_elem.set(Some(e.data())),
                            oninput: move |e| barcode.set(e.value()),
                            placeholder: "0123456789",
                        }
                    }
                    button {
                        class: "button",
                        "data-style": "outline",
                        onclick: move |_| async move {
                            barcode.set(String::new());
                            if let Some(elem) = barcode_elem() {
                                _ = elem.set_focus(true).await;
                            }
                        },

                        "Clr"
                    }
                }

                // Item display/edit
                div { display: "flex", flex_direction: "column", gap: ".5rem",
                    Label { class: "label", html_for: "item-name", "Item Name" }

                    input {
                        class: "input",
                        id: "item-name",
                        value: "{item_name}",
                        oninput: move |e| item_name.set(e.value()),
                        placeholder: "2m dipole",
                    }
                }
                div { display: "flex", flex_direction: "column", gap: ".5rem",
                    Label { class: "label", html_for: "item-desc", "Item Description" }

                    textarea {
                        class: "input",
                        id: "item-desc",
                        value: "{item_description}",
                        oninput: move |e| item_description.set(e.value()),
                    }
                }
                div { display: "flex", flex_direction: "column", gap: ".5rem",
                    Label { class: "label", html_for: "item-price", "Item Price" }

                    input {
                        class: "input",
                        id: "item-price",
                        r#type: "number",
                        step: "0.01",
                        min: "0",
                        value: "{item_price}",
                        oninput: move |e| {
                            if let Ok(price) = BigDecimal::from_str(&e.value()) {
                                item_price.set(price);
                            }
                        },
                        placeholder: "12.34",
                    }
                }
            }

        // Receipt display
        }
    }
}
