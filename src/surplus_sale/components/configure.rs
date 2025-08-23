use std::str::FromStr;

use bigdecimal::BigDecimal;
use dioxus::prelude::*;
use dioxus_primitives::{
    dialog::{DialogContent, DialogDescription, DialogRoot, DialogTitle},
    label::Label,
};
use iso_currency::Currency;

use crate::surplus_sale::types::Datafile;

pub struct ConfigurationUpdateData {
    /// The currency used
    pub currency: Currency,
    /// The decimal value of the club taking
    pub club_taking: BigDecimal,
    /// The USB vendor ID of the ESC/POS device to use
    #[cfg(feature = "escpos")]
    pub escpos_vendor: u16,
    /// The USB device ID of the ESC/POS device to use
    #[cfg(feature = "escpos")]
    pub escpos_device: u16,
}

#[derive(PartialEq, Props, Clone)]
pub struct ConfigureProps {
    open: Signal<bool>,
    on_update: EventHandler<ConfigurationUpdateData>,
    datafile: ReadOnlySignal<Datafile>,
}

#[component]
pub fn Configure(props: ConfigureProps) -> Element {
    let mut open = props.open;
    let mut currency = use_signal(|| *props.datafile.read().currency());
    #[cfg(feature = "escpos")]
    let mut escpos_vendor = use_signal(|| 0x0000);
    #[cfg(feature = "escpos")]
    let mut escpos_device = use_signal(|| 0x0000);

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
                        min: "0",
                        step: "0.1",
                        max: "100",
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

                if cfg!(feature = "escpos") {
                    ESCPOSConfigurator {
                        on_ids_changed: move |(vid, did)| {
                            #[cfg(feature = "escpos")]
                            {
                                escpos_vendor.set(vid);
                                escpos_device.set(did);
                            }
                            let _ = vid;
                            let _ = did;
                        },
                    }
                }

                button {
                    class: "button",
                    "data-style": "primary",
                    onclick: move |_| {
                        let data = ConfigurationUpdateData {
                            currency: currency(),
                            club_taking: club_taking() / 100,
                            #[cfg(feature = "escpos")]
                            escpos_vendor: escpos_vendor(),
                            #[cfg(feature = "escpos")]
                            escpos_device: escpos_device(),
                        };
                        props.on_update.call(data);
                        open.set(false);
                    },
                    "Save"
                }
            }
        }
    }
}

#[cfg(not(feature = "escpos"))]
#[component]
fn ESCPOSConfigurator(on_ids_changed: EventHandler<(u16, u16)>) -> Element {
    let _ = on_ids_changed;
    rsx! {}
}

#[cfg(feature = "escpos")]
#[component]
fn ESCPOSConfigurator(on_ids_changed: EventHandler<(u16, u16)>) -> Element {
    let mut vendor_id = use_signal(String::new);
    let mut device_id = use_signal(String::new);

    let vendor_id_translated = use_memo(move || u16::from_str_radix(&vendor_id(), 16));
    let device_id_translated = use_memo(move || u16::from_str_radix(&device_id(), 16));
    use_effect(move || {
        if let Ok(vid) = vendor_id_translated() {
            if let Ok(did) = device_id_translated() {
                on_ids_changed.call((vid, did));
            }
        }
    });

    rsx! {
        h4 { margin: 0, "ESCPOS Setup" }
        if cfg!(target_os = "windows") {
            p { margin: 0, "Vendor and Device ID can be found by looking in Device Manager" }
        } else if cfg!(target_os = "linux") {
            p { margin: 0, "Vendor and Device ID can be found by running lsusb" }
        }

        div { display: "flex", flex_direction: "row", gap: "1rem",
            div { display: "flex", flex_direction: "column", gap: ".5rem",
                Label { class: "label", html_for: "escpos-vendor", "Printer Vendor ID" }

                input {
                    class: "input",
                    id: "escpos-vendor",
                    value: "{vendor_id}",
                    oninput: move |e| vendor_id.set(e.value()),
                    placeholder: "23af",
                }
            }
            div { display: "flex", flex_direction: "column", gap: ".5rem",
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
    }
}
