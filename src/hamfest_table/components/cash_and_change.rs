use std::str::FromStr;

use bigdecimal::{BigDecimal, Zero};
use dioxus::prelude::*;
use dioxus_primitives::{
    dialog::{DialogContent, DialogRoot, DialogTitle},
    label::Label,
};

use crate::hamfest_table::types::{Receipt, ReceiptLine, TransactionMethod};

#[derive(Clone, PartialEq, Props)]
pub struct CashAndChangeDialogProps {
    open: Signal<bool>,
    receipt: Signal<Option<Receipt>>,
}

#[component]
pub fn CashAndChangeDialog(props: CashAndChangeDialogProps) -> Element {
    let CashAndChangeDialogProps {
        mut open,
        mut receipt,
    } = props;

    let total = use_memo(move || {
        if let Some(receipt) = receipt.read().as_ref() {
            receipt.total()
        } else {
            BigDecimal::zero()
        }
    });
    let mut amount_handed = use_signal(BigDecimal::zero);
    let change = use_memo(move || (amount_handed() - total()).max(BigDecimal::zero()));

    rsx! {
        DialogRoot {
            class: "dialog-backdrop",
            open: open(),
            on_open_change: move |v| open.set(v),
            DialogContent { class: "dialog",
                button {
                    class: "dialog-close",
                    aria_label: "Close",
                    tabindex: if open() { "0" } else { "-1" },
                    onclick: move |_| open.set(false),
                    "×"
                }
                DialogTitle { class: "dialog-title", "Cash and Change" }
                div { display: "flex", flex_direction: "column", gap: "1rem",
                    div {
                        display: "flex",
                        flex_direction: "column",
                        gap: ".5rem",
                        Label { class: "label", html_for: "cash-given", "Amount Given" }

                        input {
                            class: "input",
                            id: "cash-given",
                            r#type: "number",
                            step: "0.01",
                            min: "0",
                            value: "{amount_handed}",
                            oninput: move |e| {
                                if let Ok(val) = BigDecimal::from_str(&e.value()) {
                                    amount_handed.set(val);
                                }
                            },
                            placeholder: "0.00",
                        }
                    }
                    div {
                        display: "flex",
                        flex_direction: "column",
                        gap: ".5rem",
                        Label { class: "label", html_for: "change", "Change" }

                        input {
                            class: "input",
                            id: "change",
                            readonly: true,
                            value: "{change}",
                        }
                    }

                    button {
                        class: "button",
                        "data-style": "primary",
                        onclick: move |_| {
                            if let Some(receipt) = receipt.write().as_mut() {
                                receipt
                                    .lines_mut()
                                    .push(ReceiptLine::Payment {
                                        method: TransactionMethod::Cash,
                                        amount: amount_handed(),
                                    });
                                receipt
                                    .lines_mut()
                                    .push(ReceiptLine::Change {
                                        method: TransactionMethod::Cash,
                                        amount: change(),
                                    });
                                open.set(false);
                            }
                        },
                        "Complete"
                    }
                }
            }
        }
    }
}
