use std::time::Duration;

use bigdecimal::{BigDecimal, Zero};
use dioxus::prelude::*;
use dioxus_primitives::{
    dialog::{DialogContent, DialogRoot, DialogTitle},
    label::Label,
    toast::{use_toast, ToastOptions},
};

use crate::{
    hamfest_table::types::{Receipt, ReceiptLine},
    types::ESCPOSDevice,
};

#[derive(Clone, PartialEq, Props)]
pub struct PrintDialogProps {
    receipt: Signal<Option<Receipt>>,
    open: Signal<bool>,
}

#[component]
pub fn PrintDialog(props: PrintDialogProps) -> Element {
    let PrintDialogProps { receipt, mut open } = props;
    let toast_api = use_toast();

    let mut vendor_id = use_signal(String::new);
    let mut device_id = use_signal(String::new);
    let device = use_memo(move || {
        if let Ok(vid) = u16::from_str_radix(&vendor_id(), 16) {
            if let Ok(did) = u16::from_str_radix(&device_id(), 16) {
                Some(ESCPOSDevice(vid, did))
            } else {
                None
            }
        } else {
            None
        }
    });

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
                DialogTitle { class: "dialog-title", "Print" }

                div { display: "flex", flex_direction: "column", gap: "1rem",
                    if cfg!(target_os = "windows") {
                        p { margin: 0,
                            "Vendor and Device ID can be found by looking in Device Manager"
                        }
                    } else if cfg!(target_os = "linux") {
                        p { margin: 0, "Vendor and Device ID can be found by running lsusb" }
                    }

                    div { display: "flex", flex_direction: "row", gap: "1rem",
                        div {
                            display: "flex",
                            flex_direction: "column",
                            gap: ".5rem",
                            Label { class: "label", html_for: "escpos-vendor", "Printer Vendor ID" }

                            input {
                                class: "input",
                                id: "escpos-vendor",
                                value: "{vendor_id}",
                                oninput: move |e| vendor_id.set(e.value()),
                                placeholder: "23af",
                            }
                        }
                        div {
                            display: "flex",
                            flex_direction: "column",
                            gap: ".5rem",
                            Label { class: "label", html_for: "escpos-device", "Printer Device ID" }

                            input {
                                class: "input",
                                id: "escpos-device",
                                value: "{device_id}",
                                oninput: move |e| device_id.set(e.value()),
                                placeholder: "23af",
                            }
                        }
                    }
                    button {
                        class: "button",
                        "data-style": "primary",
                        disabled: device().is_none(),
                        onclick: move |_| {
                            if let Some(receipt) = receipt.read().as_ref() {
                                // SAFETY: button disabled if device is none
                                if let Err(e) = print(receipt, device().unwrap()) {
                                    toast_api
                                        .error(
                                            "Failed to print".to_string(),
                                            ToastOptions::new()
                                                .duration(Duration::from_secs(5))
                                                .description(e),
                                        );
                                } else {
                                    toast_api
                                        .success(
                                            "Printing".to_string(),
                                            ToastOptions::new().duration(Duration::from_secs(3)),
                                        );
                                }
                            }
                        },
                        "Print"
                    }
                }
            }
        }
    }
}

fn print(receipt: &Receipt, device: ESCPOSDevice) -> escpos::errors::Result<()> {
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
        .writeln("Receipt")?
        .bold(false)?
        .reset_size()?
        .justify(JustifyMode::LEFT)?
        .feed()?
        .feed()?;

    let mut grand_total = BigDecimal::zero();
    for line in receipt.lines() {
        match line {
            ReceiptLine::Item { item } => {
                grand_total += item.price();
                prn.justify(JustifyMode::LEFT)?
                    .writeln(&item.name())?
                    .justify(JustifyMode::RIGHT)?
                    .writeln(&format!("{:0.02}", item.price()))?
                    .feed()?;
            }
            ReceiptLine::Payment { method, amount } => {
                prn.size(1, 2)?
                    .justify(JustifyMode::LEFT)?
                    .writeln("Total")?
                    .justify(JustifyMode::RIGHT)?
                    .writeln(&format!("{grand_total:0.02}"))?
                    .feed()?
                    .reset_size()?;
                prn.justify(JustifyMode::LEFT)?
                    .writeln(&method.to_string())?
                    .justify(JustifyMode::RIGHT)?
                    .writeln(&format!("-{amount:0.02}"))?
                    .feed()?;
                grand_total -= amount;
            }
            ReceiptLine::Change { method, amount } => {
                grand_total += amount;
                prn.justify(JustifyMode::LEFT)?
                    .writeln(&format!("Change given via {method}"))?
                    .justify(JustifyMode::RIGHT)?
                    .writeln(&format!("{amount:0.02}"))?
                    .feed()?;
            }
        }
    }

    prn.justify(JustifyMode::CENTER)?
        .writeln(&format!("Receipt ID: {}", receipt.receipt_number()))?
        .feed()?;
    prn.justify(JustifyMode::CENTER)?
        .writeln(&format!("{}", receipt.timestamp().to_rfc2822()))?
        .feed()?;
    prn.justify(JustifyMode::CENTER)?
        .writeln("Thank you for your purchase!")?
        .feed()?;
    prn.feed()?.partial_cut()?.print()?;

    Ok(())
}
