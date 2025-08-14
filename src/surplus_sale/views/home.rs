use std::cell::RefCell;
use std::io::Cursor;
use std::time::Duration;

use chrono::Local;
use dioxus::{logger::tracing, prelude::*};
use dioxus_primitives::navbar::{Navbar, NavbarContent, NavbarItem, NavbarNav, NavbarTrigger};
use dioxus_primitives::toast::{use_toast, ToastOptions};

use crate::surplus_sale::components::LoadedFile;
use crate::surplus_sale::types::Datafile;
use crate::surplus_sale::{DatafileHandle, NeedsSaving};
use crate::Route;

use super::super::components::NavbarIcon;

pub const WARNING_DURATION: Duration = Duration::from_secs(8);
pub const ERROR_DURATION: Duration = Duration::from_secs(12);

#[component]
pub fn SurplusSale() -> Element {
    let toast_api = use_toast();
    let needs_saving = use_context_provider(|| Signal::new(NeedsSaving(false)));
    let datafile: Signal<Option<DatafileHandle>> = use_signal(|| None);

    rsx! {
        Navbar { class: "navbar", aria_label: "Navigation",

            NavbarItem {
                index: 0usize,
                class: "navbar-item",
                value: "menu".to_string(),
                to: Route::Home {},
                onclick: |_| (),
                onclick_only: true,
                on_select: {
                    let needs_saving = needs_saving.clone();
                    move |_| {
                        if needs_saving.read().0 {
                            toast_api
                                .warning(
                                    "Needs saving".to_string(),
                                    ToastOptions::new()
                                        .description("This file needs saving. Please either save it or close it, then try again.")
                                        .permanent(false)
                                        .duration(WARNING_DURATION),
                                );
                            return;
                        }
                        use_navigator().replace(Route::Home {});
                    }
                },
                "← Menu"
            }

            NavbarNav { class: "navbar-nav", index: 1usize,
                NavbarTrigger { class: "navbar-trigger",
                    "File"
                    NavbarIcon {}
                }
                NavbarContent { class: "navbar-content",
                    NavbarItem {
                        index: 0usize,
                        class: "navbar-item",
                        value: "new".to_string(),
                        to: Route::SurplusSale {},
                        onclick: |_| (),
                        onclick_only: true,
                        on_select: {
                            let mut datafile = datafile.clone();
                            let mut needs_saving = needs_saving.clone();
                            move |_| {
                                if needs_saving.read().0 {
                                    toast_api
                                        .warning(
                                            "Needs saving".to_string(),
                                            ToastOptions::new()
                                                .description("This file needs saving. Please either save it or close it, then try again.")
                                                .permanent(false)
                                                .duration(WARNING_DURATION),
                                        );
                                    return;
                                }
                                tracing::info!("Creating new...");
                                datafile.set(Some(RefCell::new(Datafile::new())));
                                needs_saving.set(NeedsSaving(true));
                            }
                        },
                        "New"
                    }
                    NavbarItem {
                        index: 1usize,
                        class: "navbar-item",
                        value: "open".to_string(),
                        to: Route::SurplusSale {},
                        onclick: |_| (),
                        onclick_only: true,
                        on_select: {
                            let mut datafile = datafile.clone();
                            let mut needs_saving = needs_saving.clone();
                            move |_| async move {
                                if needs_saving.read().0 {
                                    toast_api
                                        .warning(
                                            "Needs saving".to_string(),
                                            ToastOptions::new()
                                                .description("This file needs saving. Please either save it or close it, then try again.")
                                                .permanent(false)
                                                .duration(WARNING_DURATION),
                                        );
                                    return;
                                }
                                tracing::info!("Opening...");
                                if let Some(path) = rfd::AsyncFileDialog::new()
                                    .add_filter("TDARS auction", &["tdars_auction"])
                                    .pick_file()
                                    .await
                                {
                                    let data = path.read().await;
                                    // Parse data
                                    match rmp_serde::from_read::<_, Datafile>(Cursor::new(data)) {
                                        Ok(datafile_struct) => {
                                            datafile.set(Some(RefCell::new(datafile_struct)));
                                            needs_saving.set(NeedsSaving(false));
                                        }
                                        Err(e) => {
                                            toast_api
                                                .error(
                                                    "Failed to open".to_string(),
                                                    ToastOptions::new()
                                                        .description(format!("{e}"))
                                                        .permanent(false)
                                                        .duration(ERROR_DURATION),
                                                );
                                        }
                                    }
                                }
                            }
                        },
                        "Open..."
                    }
                    NavbarItem {
                        index: 2usize,
                        class: "navbar-item",
                        value: "save".to_string(),
                        disabled: !datafile().is_some(),
                        to: Route::SurplusSale {},
                        onclick: |_| (),
                        onclick_only: true,
                        on_select: {
                            let datafile = datafile.clone();
                            let mut needs_saving = needs_saving.clone();
                            move |_| async move {
                                tracing::info!("Saving...");
                                let datafile_struct = datafile().unwrap();
                                let data = rmp_serde::to_vec(&datafile_struct).unwrap();
                                let date = Local::now().date_naive();
                                if let Some(path) = rfd::AsyncFileDialog::new()
                                    .add_filter("TDARS auction", &["tdars_auction"])
                                    .set_file_name(format!("{date}.tdars_auction"))
                                    .save_file()
                                    .await
                                {
                                    if let Err(e) = path.write(&data).await {
                                        toast_api
                                            .error(
                                                "Failed to save".to_string(),
                                                ToastOptions::new()
                                                    .description(format!("{e}"))
                                                    .permanent(false)
                                                    .duration(ERROR_DURATION),
                                            );
                                    } else {
                                        needs_saving.set(NeedsSaving(false));
                                    }
                                }
                            }
                        },
                        "Save"
                    }
                    NavbarItem {
                        index: 3usize,
                        class: "navbar-item",
                        value: "close".to_string(),
                        disabled: !datafile().is_some(),
                        to: Route::SurplusSale {},
                        onclick: |_| (),
                        onclick_only: true,
                        on_select: {
                            let mut datafile = datafile.clone();
                            let mut needs_saving = needs_saving.clone();
                            move |_| async move {
                                if needs_saving.read().0 {
                                    let response = rfd::AsyncMessageDialog::new()
                                        .set_title("This file needs saving")
                                        .set_description("This file needs saving. Do you really want to close without saving this file?")
                                        .set_level(rfd::MessageLevel::Warning)
                                        .set_buttons(rfd::MessageButtons::YesNo)
                                        .show()
                                        .await;
                                    if response != rfd::MessageDialogResult::Yes {
                                        return;
                                    }
                                }
                                tracing::info!("Closing...");
                                datafile.set(None);
                                needs_saving.set(NeedsSaving(false));
                            }
                        },
                        "Close"
                    }
                }
            }

            NavbarNav {
                class: "navbar-nav",
                index: 2usize,
                disabled: !datafile().is_some(),
                NavbarTrigger { class: "navbar-trigger",
                    "Export"
                    NavbarIcon {}
                }
                NavbarContent { class: "navbar-content",
                    NavbarItem {
                        index: 0usize,
                        class: "navbar-item",
                        value: "New".to_string(),
                        to: Route::SurplusSale {},
                        onclick: |_| (),
                        onclick_only: true,
                        on_select: |_| tracing::info!("Exporting..."),
                        "Transaction Ledger"
                    }
                }
            }
        }

        h2 { font_size: "1rem", "Surplus Sale" }

        if datafile().is_some() {
            LoadedFile { loaded_file: datafile().unwrap() }
        } else {
            "Nothing open..."
        }
    }
}
