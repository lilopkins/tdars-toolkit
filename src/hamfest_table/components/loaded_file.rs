use std::{rc::Rc, str::FromStr};

use bigdecimal::{BigDecimal, Zero};
use dioxus::{logger::tracing, prelude::*};
use dioxus_primitives::{
    label::Label,
    scroll_area::{ScrollArea, ScrollDirection},
};

use crate::{
    hamfest_table::{
        components::CashAndChangeDialog,
        types::{Datafile, Item, Receipt, ReceiptLine, TransactionMethod},
    },
    Route,
};

#[component]
pub fn LoadedFile(datafile: MappedMutSignal<Datafile, Signal<Option<Datafile>>>) -> Element {
    let mut barcode = use_signal(String::new);
    let mut barcode_elem: Signal<Option<Rc<MountedData>>> = use_signal(|| None);

    let mut receipt: Signal<Option<Receipt>> = use_signal(|| None);
    let mut receipt_selected = use_signal(|| usize::MAX);
    let mut ensure_receipt = move || {
        if let Some(receipt_inner) = receipt().as_ref() {
            // If receipt is paid off, i.e. total becomes zero, save receipt and create new
            if receipt_inner.total() == BigDecimal::zero() {
                // Save Receipt
                datafile.write().receipts_mut().push(receipt_inner.clone());
                receipt.set(Some(Receipt::new()));
                receipt_selected.set(usize::MAX);
            }
        } else {
            // If no receipt exists, create one
            receipt.set(Some(Receipt::new()));
            receipt_selected.set(usize::MAX);
        }
    };

    let mut item = use_signal(Item::default);
    use_effect(move || {
        // Reload item details from barcode
        let barcode = barcode();
        receipt_selected.set(usize::MAX);
        if barcode.is_empty() {
            item.set(Item::default());
        } else {
            // Try to fetch item from barcode
            if let Some(df_item) = datafile
                .peek()
                .items()
                .iter()
                .find(|i| *i.barcode() == barcode)
            {
                tracing::info!("Barcode changed and item found! {df_item}");
                item.set(df_item.clone());
            } else {
                tracing::info!("Barcode changed to nonexistant item.");
                item.set(Item::default());
            }
        }
    });
    use_effect(move || {
        // Update item when details changed/created
        let barcode = barcode.peek();
        let item = item();
        let item_name = item.name().clone();
        let item_description = item.description().clone();
        let item_price = item.price().clone();
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

    let mut cash_and_change_dialog_open = use_signal(|| false);

    rsx! {
        div {
            display: "flex",
            gap: "1rem",
            height: "calc(100vh - 2 * var(--page-margin))",
            div {
                display: "flex",
                flex_direction: "column",
                gap: "1rem",
                flex_grow: 1,
                Link { to: Route::Home {},
                    button { class: "button", "data-style": "outline", "← Main Menu" }
                }
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
                        "data-style": "primary",
                        onclick: move |_| async move {
                            barcode.set(String::new());
                            if let Some(elem) = barcode_elem() {
                                _ = elem.set_focus(true).await;
                            }
                        },

                        "Clear"
                    }
                    button {
                        class: "button",
                        "data-style": "primary",
                        disabled: barcode().is_empty() && receipt_selected() == usize::MAX,
                        onclick: move |_| async move {
                            if receipt_selected() == usize::MAX {
                                // No receipt item selected, add item to receipt
                                ensure_receipt();
                                if let Some(receipt) = receipt.write().as_mut() {
                                    // Only add to receipt if it's not already on there.
                                    if !receipt
                                        .lines()
                                        .iter()
                                        .filter_map(|l| match l {
                                            ReceiptLine::Item { item } => Some(item),
                                            _ => None,
                                        })
                                        .any(|i| *i.barcode() == barcode.read().clone())
                                    {
                                        receipt.lines_mut().push(ReceiptLine::Item { item: item() });
                                    }
                                }
                                barcode.set(String::new());
                                if let Some(elem) = barcode_elem() {
                                    _ = elem.set_focus(true).await;
                                }
                            } else {
                                // Receipt item selected, remove that index
                                if let Some(receipt) = receipt.write().as_mut() {
                                    let idx = receipt_selected();
                                    tracing::info!("Removing line {idx} from receipt");
                                    receipt.lines_mut().remove(idx);
                                    receipt_selected.set(usize::MAX);
                                }
                            }
                        },

                        if receipt_selected() == usize::MAX {
                            "→"
                        } else {
                            "←"
                        }
                    }
                }

                // Item display/edit
                div { display: "flex", flex_direction: "column", gap: ".5rem",
                    Label { class: "label", html_for: "item-name", "Item Name" }

                    input {
                        class: "input",
                        id: "item-name",
                        value: "{item.read().name()}",
                        oninput: move |e| {
                            item.write().set_name(e.value());
                        },
                    }
                }
                div {
                    display: "flex",
                    flex_direction: "column",
                    gap: ".5rem",
                    flex_grow: 1,
                    Label { class: "label", html_for: "item-desc", "Item Description" }

                    textarea {
                        class: "input",
                        id: "item-desc",
                        flex_grow: 1,
                        value: "{item.read().description()}",
                        oninput: move |e| {
                            item.write().set_description(e.value());
                        },
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
                        value: "{item.read().price()}",
                        oninput: move |e| {
                            if let Ok(price) = BigDecimal::from_str(&e.value()) {
                                item.write().set_price(price);
                            }
                        },
                        placeholder: "12.34",
                    }
                }
            }

            // Receipt display
            div {
                display: "flex",
                flex_direction: "column",
                gap: "1rem",
                flex_grow: 1,
                div { flex_grow: 1,
                    ScrollArea {
                        direction: ScrollDirection::Vertical,
                        width: "100%",
                        height: "100%",
                        // really dirty trick to force a reflow on selection change
                        "data-selected-item": "{receipt_selected()}",

                        if let Some(receipt) = receipt.read().as_ref() {
                            p {
                                font_size: "200%",
                                font_family: "monospace",
                                text_align: "center",
                                "Receipt"
                            }
                            p {
                                font_family: "monospace",
                                text_align: "center",
                                "{receipt.timestamp().to_rfc2822()}"
                            }
                            p {
                                font_family: "monospace",
                                text_align: "center",
                                "Receipt: {receipt.receipt_number()}"
                            }
                            for (idx , line) in receipt.lines().iter().enumerate() {
                                div { onclick: move |_| receipt_selected.set(idx),
                                    ReceiptLineComponent {
                                        line: line.clone(),
                                        idx,
                                        selected: receipt_selected.clone(),
                                    }
                                }
                            }
                            p {
                                font_family: "monospace",
                                text_align: "end",
                                font_weight: "bold",
                                "Total: {receipt.total():0.02}"
                            }
                        } else {
                            p { font_family: "monospace", "No receipt" }
                        }
                    }
                }
                div { display: "flex", gap: ".5rem",
                    button {
                        class: "button",
                        "data-style": "primary",
                        onclick: move |_| cash_and_change_dialog_open.set(true),
                        "Cash"
                    }
                    CashAndChangeDialog { receipt, open: cash_and_change_dialog_open }
                    button {
                        class: "button",
                        "data-style": "primary",
                        onclick: move |_| {
                            if let Some(receipt) = receipt.write().as_mut() {
                                let total = receipt.total();
                                receipt
                                    .lines_mut()
                                    .push(ReceiptLine::Payment {
                                        method: TransactionMethod::Card,
                                        amount: total,
                                    })
                            }
                        },
                        "Card"
                    }
                    button {
                        class: "button",
                        "data-style": "primary",
                        onclick: move |_| {
                            if let Some(receipt) = receipt.write().as_mut() {
                                let total = receipt.total();
                                receipt
                                    .lines_mut()
                                    .push(ReceiptLine::Payment {
                                        method: TransactionMethod::BankTransfer,
                                        amount: total,
                                    })
                            }
                        },
                        "Bank Transfer"
                    }
                    button {
                        class: "button",
                        "data-style": "primary",
                        onclick: move |_| {
                            if let Some(receipt) = receipt.write().as_mut() {
                                let total = receipt.total();
                                receipt
                                    .lines_mut()
                                    .push(ReceiptLine::Payment {
                                        method: TransactionMethod::Cheque,
                                        amount: total,
                                    })
                            }
                        },
                        "Cheque"
                    }
                    {print_dialog(receipt.clone())}
                    button {
                        class: "button",
                        "data-style": "primary",
                        onclick: move |_| {
                            receipt.set(None);
                            receipt_selected.set(usize::MAX)
                        },
                        "Void Receipt"
                    }
                }
            }
        }
    }
}

#[component]
fn ReceiptLineComponent(line: ReceiptLine, idx: usize, selected: Signal<usize>) -> Element {
    let bg = if idx == selected() {
        "rgba(0, 0, 0, 0.15)"
    } else {
        "rgba(0, 0, 0, 0)"
    };
    match line {
        ReceiptLine::Item { item } => {
            rsx! {
                div {
                    font_family: "monospace",
                    background_color: bg,
                    padding: "6px",

                    p { "{item.name()}" }
                    p { text_align: "end", "{item.price():0.02}" }
                }
            }
        }
        ReceiptLine::Payment { method, amount } => {
            rsx! {
                div {
                    font_family: "monospace",
                    background_color: bg,
                    padding: "6px",

                    p { "{method}" }
                    p { text_align: "end", "-{amount:0.02}" }
                }
            }
        }
        ReceiptLine::Change { method, amount } => {
            rsx! {
                div {
                    font_family: "monospace",
                    background_color: bg,
                    padding: "6px",

                    p { "Change via {method}" }
                    p { text_align: "end", "{amount:0.02}" }
                }
            }
        }
    }
}

#[cfg(feature = "escpos")]
pub fn print_dialog(receipt: Signal<Option<Receipt>>) -> Element {
    use crate::hamfest_table::components::PrintDialog;

    let mut open = use_signal(|| false);
    rsx! {
        button {
            class: "button",
            "data-style": "primary",
            disabled: receipt.read().is_none(),
            onclick: move |_| open.set(true),

            "Print"
        }
        PrintDialog { receipt, open }
    }
}

#[cfg(not(feature = "escpos"))]
pub fn print_dialog(_receipt: Signal<Receipt>) -> Element {
    rsx! {}
}
