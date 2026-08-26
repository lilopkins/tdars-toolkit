#[cfg(feature = "escpos")]
use std::time::Duration;
use std::{cmp::Ordering, str::FromStr};

use bigdecimal::{BigDecimal, Zero};
use dioxus::prelude::*;
use dioxus_primitives::{
    label::Label,
    separator::Separator,
    toast::{use_toast, ToastOptions},
};

#[cfg(feature = "escpos")]
use crate::surplus_sale::types::Item;
use crate::{
    components::CallsignEntry,
    surplus_sale::{
        types::{Datafile, ReconcileMethod},
        NeedsSaving,
    },
    types::Callsign,
};

#[derive(Clone, PartialEq)]
struct OutstandingBoughtItem {
    lot_number: String,
    description: String,
    amount: BigDecimal,
}

#[component]
pub fn Reconciliation() -> Element {
    let toast_api = use_toast();
    let mut datafile: Signal<Datafile> = use_context();
    let mut needs_saving: Signal<NeedsSaving> = use_context();
    let callsign = use_signal(Callsign::default);
    let mut reconcile_amount = use_signal(BigDecimal::zero);

    #[cfg(feature = "escpos")]
    let escpos_device: Signal<crate::types::ESCPOSDevice> = use_context();

    let sym = use_memo(move || datafile.read().currency().symbol());
    let unmatched_liability = use_memo(move || {
        datafile
            .read()
            .callsign_liabilities()
            .get(&callsign())
            .map_or_else(BigDecimal::zero, |liability| liability.amount().clone())
    });
    let items_sold = use_memo(move || {
        datafile
            .read()
            .items()
            .iter()
            .filter(|i| {
                *i.seller_callsign() == callsign()
                    && i.sold_details()
                        .as_ref()
                        .is_some_and(|s| s.seller_reconciled().is_none())
            })
            .cloned()
            .collect::<Vec<_>>()
    });
    let items_bought = use_memo(move || outstanding_bought_items(&datafile.read(), &callsign()));

    // + => callsign pays club
    // - => club pays callsign
    let total = use_memo(move || {
        let mut total = unmatched_liability();
        for item in items_bought.read().iter() {
            total += item.amount.clone();
        }
        for item in items_sold.read().iter() {
            if let Some(sold) = item.sold_details() {
                total -= sold.hammer_price() * (1 - datafile().club_taking());
            }
        }
        total
    });
    use_effect(move || reconcile_amount.set(total().abs()));

    let mut reconcile = move |method| {
        let amt = if total() < BigDecimal::zero() {
            -reconcile_amount()
        } else {
            reconcile_amount()
        };
        let change = datafile.write().reconcile(&callsign(), amt, method);
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

            CallsignEntry {
                suggestion_source: datafile.read().callsigns().clone(),
                value: callsign,
            }

            div { display: "flex", flex_direction: "row", gap: ".6rem",

                div { display: "flex", flex_direction: "column", gap: ".5rem",
                    Label { class: "label", html_for: "reconcile-amount", "Amount to Reconcile" }
                    input {
                        class: "input",
                        id: "reconcile-amount",
                        placeholder: "3.50",
                        style: "width: 6em",
                        value: "{reconcile_amount():0.02}",
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
                        disabled: total() == BigDecimal::zero(),
                        "data-style": "primary",
                        onclick: move |_| reconcile(ReconcileMethod::Cash),
                        "Cash"
                    }
                }
                div { align_content: "end", margin_left: ".4rem",
                    button {
                        class: "button",
                        disabled: total() == BigDecimal::zero(),
                        "data-style": "primary",
                        onclick: move |_| reconcile(ReconcileMethod::BankTransfer {
                            seen: true,
                        }),
                        "Bank Transfer (Seen)"
                    }
                }
                div { align_content: "end", margin_left: ".4rem",
                    button {
                        class: "button",
                        disabled: total() == BigDecimal::zero(),
                        "data-style": "primary",
                        onclick: move |_| reconcile(ReconcileMethod::BankTransfer {
                            seen: false,
                        }),
                        "Bank Transfer (Unseen)"
                    }
                }
                div { align_content: "end", margin_left: ".4rem",
                    button {
                        class: "button",
                        disabled: total() == BigDecimal::zero(),
                        "data-style": "primary",
                        onclick: move |_| reconcile(ReconcileMethod::Postpone),
                        "Reconcile with Postponed Payment"
                    }
                }
                div { align_content: "end", margin_left: ".4rem",
                    button {
                        class: "button",
                        disabled: total() >= BigDecimal::zero() && reconcile_amount() <= total(),
                        "data-style": "primary",
                        onclick: move |_| reconcile(ReconcileMethod::Donation),
                        if total() > BigDecimal::zero() {
                            "Change to Club"
                        } else {
                            "Revenue to Club"
                        }
                    }
                }
            }

            p { margin: 0,
                "This individual "
                match total().cmp(&BigDecimal::zero()) {
                    Ordering::Greater => "owes the club the displayed amount.",
                    Ordering::Equal => "has nothing to reconcile.",
                    Ordering::Less => "is owed by the club the displayed amount.",
                }
            }

            Separator { class: "separator", horizontal: true, decorative: true }

            if cfg!(feature = "escpos") {
                button {
                    class: "button",
                    "data-style": "primary",
                    onclick: move |_| {
                        #[cfg(feature = "escpos")]
                        {
                            match print_receipt(
                                escpos_device(),
                                &callsign(),
                                &unmatched_liability(),
                                items_sold.read().as_ref(),
                                items_bought.read().as_ref(),
                                datafile.read().club_taking(),
                            ) {
                                Ok(()) => {
                                    toast_api
                                        .info(
                                            "Receipt printing".to_string(),
                                            ToastOptions::new()
                                                .permanent(false)
                                                .duration(Duration::from_secs(3)),
                                        );
                                }
                                Err(e) => {
                                    toast_api
                                        .error(
                                            "Failed to print".to_string(),
                                            ToastOptions::new()
                                                .permanent(false)
                                                .duration(Duration::from_secs(5))
                                                .description(format!("{e}")),
                                        );
                                }
                            }
                        }
                    },
                    "Print Receipt"
                }
            }

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
                    if !unmatched_liability().is_zero() {
                            tr {
                                td {}
                                td {
                                    em { "Unpaid owing" }
                                }
                                td { "-{sym} {unmatched_liability():0.02}" }
                                td { "-{sym} {unmatched_liability():0.02}" }
                            }
                    }
                    for item in items_bought.read().iter() {
                        tr {
                            td { "{item.lot_number}" }
                            td { "{item.description}" }
                            td { "-{sym} {item.amount:0.02}" }
                            td { "-{sym} {item.amount:0.02}" }
                        }
                    }
                    for item in &items_sold() {
                        tr {
                            td { "{item.lot_number()}" }
                            td { "{item.description()}" }
                            if let Some(sold) = item.sold_details() {
                                td { "{sym} {sold.hammer_price():0.02}" }
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
                                    em {
                                        "-{sym} {(sold.hammer_price() * datafile().club_taking()):0.02}"
                                    }
                                }
                                td {
                                    "{sym} {(sold.hammer_price() * (1 - datafile().club_taking())):0.02}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "escpos")]
fn print_receipt(
    device: crate::types::ESCPOSDevice,
    callsign: &Callsign,
    liability: &BigDecimal,
    sold: &Vec<Item>,
    bought: &Vec<OutstandingBoughtItem>,
    club_taking: &BigDecimal,
) -> escpos::errors::Result<()> {
    use escpos::{
        driver::UsbDriver,
        printer::Printer,
        printer_options::PrinterOptions,
        utils::{JustifyMode, Protocol},
    };

    let driver = UsbDriver::open(device.0, device.1, None, None)?;
    let mut prn = Printer::new(driver, Protocol::default(), Some(PrinterOptions::default()));
    let prn = prn
        .init()?
        .reset()?
        .smoothing(true)?
        .bold(true)?
        .size(2, 2)?
        .justify(JustifyMode::CENTER)?
        .writeln("Surplus Sale")?
        .bold(false)?
        .size(2, 1)?
        .writeln(callsign.callsign())?
        .reset_size()?
        .justify(JustifyMode::LEFT)?
        .feed()?
        .feed()?;

    let mut grand_total = BigDecimal::zero();

    if !liability.is_zero() {
        grand_total += liability;
        prn.justify(JustifyMode::LEFT)?
            .writeln("Unpaid amounts")?
            .justify(JustifyMode::RIGHT)?
            .writeln(&format!("{liability:0.02}"))?
            .feed()?;
    }

    for item in bought {
        grand_total += &item.amount;
        prn.justify(JustifyMode::LEFT)?
            .writeln(&item.description)?
            .justify(JustifyMode::RIGHT)?
            .writeln(&format!("{:0.02}", item.amount))?
            .feed()?;
    }

    for item in sold {
        if let Some(sold) = item.sold_details() {
            grand_total -= sold.hammer_price() * (1 - club_taking);
            prn.justify(JustifyMode::LEFT)?
                .writeln(item.description())?
                .justify(JustifyMode::RIGHT)?
                .writeln(&format!("-{:0.02}", sold.hammer_price()))?
                .justify(JustifyMode::LEFT)?
                .writeln("  (less club taking)")?
                .justify(JustifyMode::RIGHT)?
                .writeln(&format!("{:0.02}", sold.hammer_price() * club_taking))?
                .feed()?;
        }
    }

    prn.size(1, 2)?
        .justify(JustifyMode::LEFT)?
        .writeln("Grand Total")?
        .justify(JustifyMode::RIGHT)?
        .writeln(&format!("{grand_total:0.02}"))?
        .feed()?
        .feed()?;

    prn.partial_cut()?.print()?;

    Ok(())
}

fn outstanding_bought_items(
    datafile: &Datafile,
    callsign: &Callsign,
) -> Vec<OutstandingBoughtItem> {
    let tracked = datafile
        .callsign_liabilities()
        .get(callsign)
        .map(|liability| {
            liability
                .item_payments()
                .iter()
                .filter(|item| !item.remaining().is_zero())
                .map(|item| {
                    (
                        item.lot_number().clone(),
                        OutstandingBoughtItem {
                            lot_number: item.lot_number().clone(),
                            description: item.description().clone(),
                            amount: item.remaining().clone(),
                        },
                    )
                })
                .collect::<std::collections::HashMap<_, _>>()
        })
        .unwrap_or_default();
    let mut tracked = tracked;
    let mut outstanding = vec![];

    for item in datafile.items() {
        if let Some(sold) = item.sold_details() {
            if sold.buyer_callsign() != callsign {
                continue;
            }

            if let Some(item) = tracked.remove(item.lot_number()) {
                outstanding.push(item);
            } else if sold.buyer_reconciled().is_none() {
                outstanding.push(OutstandingBoughtItem {
                    lot_number: item.lot_number().clone(),
                    description: item.description().clone(),
                    amount: sold.hammer_price().clone(),
                });
            }
        }
    }

    outstanding.extend(tracked.into_values());
    outstanding
}
