use std::{cmp::Ordering, str::FromStr};

use bigdecimal::{BigDecimal, Zero};
use dioxus::prelude::*;
use dioxus_primitives::{
    label::Label,
    separator::Separator,
    toast::{use_toast, ToastOptions},
};

use crate::{
    components::CallsignEntry,
    surplus_sale::{types::Datafile, NeedsSaving},
    types::Callsign,
};

#[component]
pub fn Reconciliation() -> Element {
    let toast_api = use_toast();
    let mut datafile: Signal<Datafile> = use_context();
    let mut needs_saving: Signal<NeedsSaving> = use_context();
    let callsign = use_signal(Callsign::default);
    let mut reconcile_amount = use_signal(BigDecimal::zero);

    let liability = use_memo(move || {
        datafile.read().callsign_liabilities().get(&callsign()).cloned().clone()
    });
    let items_sold = use_memo(move || {
        datafile
            .read()
            .items()
            .iter()
            .filter(|i| {
                *i.seller_callsign() == callsign()
                    && i.sold_details().as_ref().is_some_and(|s| {
                        !s.seller_reconciled()
                    })
            })
            .cloned()
            .collect::<Vec<_>>()
    });
    let items_bought = use_memo(move || {
        datafile
            .read()
            .items()
            .iter()
            .filter(|i| {
                i.sold_details().as_ref().is_some_and(|s| {
                    *s.buyer_callsign() == callsign() && !s.buyer_reconciled()
                })
            })
            .cloned()
            .collect::<Vec<_>>()
    });

    // + => callsign pays club
    // - => club pays callsign
    let total = use_memo(move || {
        let mut total = liability().unwrap_or_else(BigDecimal::zero);
        for item in &items_bought() {
            if let Some(sold) = item.sold_details() {
                total -= sold.hammer_price();
            }
        }
        for item in &items_sold() {
            if let Some(sold) = item.sold_details() {
                total += sold.hammer_price() * (1 - datafile().club_taking());
            }
        }
        total
    });
    use_effect(move || reconcile_amount.set(total()));

    let mut reconcile = move |to_club| {
        let amt = if total() < BigDecimal::zero() {
            -reconcile_amount()
        } else {
            reconcile_amount()
        };
        let change = datafile.write().reconcile(callsign(), amt, to_club);
        if !change.is_zero() {
            toast_api.info(
                format!("Change for {callsign}"),
                ToastOptions::new().description(format!("{change:0.02} to be given back")),
            );
        }
        needs_saving.set(NeedsSaving(true));
    };

    rsx! {
        div { display: "flex", flex_direction: "column", gap: "1rem",

            div { display: "flex", flex_direction: "row", gap: ".6rem",

                CallsignEntry {
                    suggestion_source: datafile.read().callsigns().clone(),
                    value: callsign,
                }

                Separator {
                    class: "separator",
                    decorative: true,
                    horizontal: false,
                    height: "50px",
                    margin_left: "1.5rem",
                    margin_right: "1.5rem",
                }

                div { display: "flex", flex_direction: "column", gap: ".5rem",
                    Label { class: "label", html_for: "reconcile-amount", "Amount to Reconcile" }
                    input {
                        class: "input",
                        id: "reconcile-amount",
                        placeholder: "3.50",
                        style: "width: 6em",
                        value: "{reconcile_amount().abs():0.02}",
                        onchange: move |e| {
                            if let Ok(amt) = BigDecimal::from_str(&e.value()) {
                                reconcile_amount.set(amt);
                            }
                        },
                    }
                }
                div { align_content: "end", margin_left: ".4rem",
                    button {
                        class: "button",
                        "data-style": "primary",
                        onclick: move |_| reconcile(false),
                        "Reconcile"
                    }
                }
                div { align_content: "end", margin_left: ".4rem",
                    button {
                        class: "button",
                        "data-style": "primary",
                        onclick: move |_| reconcile(true),
                        "Revenue to Club"
                    }
                }
            }

            p { margin: 0,
                "This individual "
                match total().cmp(&BigDecimal::zero()) {
                    Ordering::Less => "owes the club the displayed amount.",
                    Ordering::Equal => "has nothing to reconcile.",
                    Ordering::Greater => "is owed by the club the displayed amount.",
                }
            }

            Separator { class: "separator", horizontal: true, decorative: true }

            table { class: "table",
                thead {
                    tr {
                        th { "Lot number" }
                        th { "Item description" }
                        th { "Amount" }
                        th { "Line total" }
                    }
                }
                tbody {
                    if let Some(liability) = liability() {
                        tr {
                            td {}
                            td { em { "Unpaid owing" } }
                            td { "{liability:0.02}" }
                            td { "{liability:0.02}" }
                        }
                    }
                    for item in &items_bought() {
                        tr {
                            td { "{item.lot_number()}" }
                            td { "{item.description()}" }
                            if let Some(sold) = item.sold_details() {
                                td { "({sold.hammer_price():0.02})" }
                                td { "({sold.hammer_price():0.02})" }
                            } else {
                                td { colspan: 3, "not sold" }
                            }
                        }
                    }
                    for item in &items_sold() {
                        tr {
                            td { "{item.lot_number()}" }
                            td { "{item.description()}" }
                            if let Some(sold) = item.sold_details() {
                                td { "{sold.hammer_price():0.02}" }
                                td {}
                            } else {
                                td { colspan: 3, "not sold" }
                            }
                        }
                        if let Some(sold) = item.sold_details() {
                            tr {
                                td { colspan: 2, text_align: "right",
                                    em { "less club taking:" }
                                }
                                td {
                                    em { "({(sold.hammer_price() * datafile().club_taking()):0.02})" }
                                }
                                td { "{(sold.hammer_price() * (1 - datafile().club_taking())):0.02}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
