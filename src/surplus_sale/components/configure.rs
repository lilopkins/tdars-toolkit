use std::str::FromStr;

use bigdecimal::BigDecimal;
use dioxus::prelude::*;
use dioxus_primitives::{
    dialog::{DialogContent, DialogDescription, DialogRoot, DialogTitle},
    label::Label,
};
use iso_currency::Currency;

use crate::surplus_sale::types::Datafile;

#[derive(PartialEq, Props, Clone)]
pub struct ConfigureProps {
    open: Signal<bool>,
    on_update: EventHandler<(Currency, BigDecimal)>,
    datafile: ReadOnlySignal<Datafile>,
}

#[component]
pub fn Configure(props: ConfigureProps) -> Element {
    let mut open = props.open;
    let mut currency = use_signal(|| *props.datafile.read().currency());

    let mut club_taking = use_signal(|| props.datafile.read().club_taking().clone() * 100);
    let mut club_taking_warning = use_signal(|| false);

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
                DialogTitle { class: "dialog-title", "Configure Surplus Sale" }
                DialogDescription { class: "dialog-description", "Configure the surplus sale's parameters" }

                div { display: "flex", flex_direction: "column", gap: ".5rem",
                    Label { class: "label", html_for: "club-percentage", "Club Takings (Percentage)" }

                    input {
                        class: "input",
                        r#type: "number",
                        id: "club-percentage",
                        value: "{club_taking}",
                        oninput: move |e| {
                            if let Ok(taking) = BigDecimal::from_str(&e.value()) {
                                club_taking.set(taking);
                                club_taking_warning.set(false);
                            } else {
                                club_taking_warning.set(true);
                            }
                        },
                        placeholder: "10",
                    }
                    if club_taking_warning() {
                        p { font_size: ".5em", margin_top: 0,

                            "The value you have entered is invalid and will be ignored!"
                        }
                    }
                }

                div { display: "flex", flex_direction: "column", gap: ".5rem",
                    Label { class: "label", html_for: "currency", "Currency" }

                    input {
                        class: "input",
                        id: "currency",
                        value: "{currency().code()}",
                        oninput: move |e| {
                            if let Some(cur) = Currency::from_code(&e.value().to_ascii_uppercase()) {
                                currency.set(cur);
                            }
                        },
                        placeholder: "Enter an ISO 3-letter currency code, e.g. GBP, EUR, USD",
                    }
                    p { font_size: ".5em", margin_top: 0,
                        "Selected currency: {currency} ({currency().code()})"
                    }
                }

                button {
                    class: "button",
                    "data-style": "primary",
                    onclick: move |_| {
                        props.on_update.call((currency(), club_taking() / 100));
                        open.set(false);
                    },
                    "Save"
                }
            }
        }
    }
}
