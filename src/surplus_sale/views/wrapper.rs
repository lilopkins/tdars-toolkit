use std::io::Cursor;
use std::time::Duration;

use chrono::Local;
use dioxus::{logger::tracing, prelude::*};
use dioxus_primitives::navbar::{Navbar, NavbarContent, NavbarItem, NavbarNav, NavbarTrigger};
use dioxus_primitives::toast::{use_toast, ToastOptions};

use crate::surplus_sale::components::LoadedFile;
use crate::surplus_sale::export::export;
use crate::surplus_sale::types::Datafile;
use crate::surplus_sale::NeedsSaving;
use crate::Route;

pub const INFO_DURATION: Duration = Duration::from_secs(5);
pub const WARNING_DURATION: Duration = Duration::from_secs(8);
pub const ERROR_DURATION: Duration = Duration::from_secs(12);

#[component]
pub fn SurplusSale() -> Element {
    let toast_api = use_toast();
    let mut needs_saving = use_context_provider(|| Signal::new(NeedsSaving(false)));
    let mut datafile: Signal<Datafile> = use_signal(Datafile::new);
    let mut datafile_open = use_signal(|| false);
    let mut configure_open = use_signal(|| false);

    rsx! {
        Navbar { class: "navbar", aria_label: "Navigation",

            NavbarItem {
                index: 0usize,
                class: "navbar-item",
                value: "menu".to_string(),
                to: Route::Home {},
                onclick: |_| (),
                onclick_only: true,
                on_select: move |_| {
                    if needs_saving.read().0 {
                        toast_api
                            .warning(
                                "Needs saving".to_string(),
                                ToastOptions::new()
                                    .description(
                                        "This file needs saving. Please either save it or close it, then try again.",
                                    )
                                    .permanent(false)
                                    .duration(WARNING_DURATION),
                            );
                        return;
                    }
                    use_navigator().replace(Route::Home {});
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
                        on_select: move |_| {
                            if needs_saving.read().0 {
                                toast_api
                                    .warning(
                                        "Needs saving".to_string(),
                                        ToastOptions::new()
                                            .description(
                                                "This file needs saving. Please either save it or close it, then try again.",
                                            )
                                            .permanent(false)
                                            .duration(WARNING_DURATION),
                                    );
                                return;
                            }
                            tracing::info!("Creating new...");
                            datafile.set(Datafile::new());
                            datafile_open.set(true);
                            needs_saving.set(NeedsSaving(true));
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
                        on_select: move |_| async move {
                            if needs_saving.read().0 {
                                toast_api
                                    .warning(
                                        "Needs saving".to_string(),
                                        ToastOptions::new()
                                            .description(
                                                // Parse data
                                                "This file needs saving. Please either save it or close it, then try again.",
                                            )
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
                                match rmp_serde::from_read::<_, Datafile>(Cursor::new(data)) {
                                    Ok(datafile_struct) => {
                                        datafile.set(datafile_struct);
                                        datafile_open.set(true);
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
                        },
                        "Open..."
                    }
                    NavbarItem {
                        index: 2usize,
                        class: "navbar-item",
                        value: "save".to_string(),
                        disabled: !datafile_open(),
                        to: Route::SurplusSale {},
                        onclick: |_| (),
                        onclick_only: true,
                        on_select: move |_| async move {
                            tracing::info!("Saving...");

                            #[allow(
                                // Button is disabled when this doesn't unwrap
                                clippy::unwrap_used,
                                reason = "the ability to serialise the datafile is guaranteed"
                            )]
                            let data = rmp_serde::to_vec(&datafile()).unwrap();
                            let date = Local::now().date_naive();
                            if let Some(handle) = rfd::AsyncFileDialog::new()
                                .add_filter("TDARS auction", &["tdars_auction"])
                                .set_file_name(format!("{date}.tdars_auction"))
                                .save_file()
                                .await
                            {
                                if let Err(e) = handle.write(&data).await {
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

                        },
                        "Save"
                    }
                    NavbarItem {
                        index: 3usize,
                        class: "navbar-item",
                        value: "configure".to_string(),
                        disabled: !datafile_open(),
                        to: Route::SurplusSale {},
                        onclick: |_| (),
                        onclick_only: true,
                        on_select: move |_| configure_open.set(true),
                        "Configuration"
                    }
                    NavbarItem {
                        index: 4usize,
                        class: "navbar-item",
                        value: "close".to_string(),
                        disabled: !datafile_open(),
                        to: Route::SurplusSale {},
                        onclick: |_| (),
                        onclick_only: true,
                        on_select: move |_| async move {
                            if needs_saving.read().0 {
                                let response = rfd::AsyncMessageDialog::new()
                                    .set_title("This file needs saving")
                                    .set_description(
                                        "This file needs saving. Do you really want to close without saving this file?",
                                    )
                                    .set_level(rfd::MessageLevel::Warning)
                                    .set_buttons(rfd::MessageButtons::YesNo)
                                    .show()
                                    .await;
                                if response != rfd::MessageDialogResult::Yes {
                                    return;
                                }
                            }
                            tracing::info!("Closing...");
                            datafile.set(Datafile::new());
                            datafile_open.set(false);
                            needs_saving.set(NeedsSaving(false));
                        },
                        "Close"
                    }
                }
            }

            NavbarNav {
                class: "navbar-nav",
                index: 2usize,
                disabled: !datafile_open(),
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
                        on_select: move |_| async move {
                            tracing::info!("Exporting...");
                            if let Some(handle) = rfd::AsyncFileDialog::new()
                                .add_filter("Excel Workbook", &["xlsx"])
                                .save_file()
                                .await
                            {
                                match export(datafile()) {
                                    Err(e) => {
                                        toast_api
                                            .error(
                                                "Failed to export".to_string(),
                                                ToastOptions::new()
                                                    .description(format!("{e}"))
                                                    .permanent(false)
                                                    .duration(ERROR_DURATION),
                                            );
                                    }
                                    Ok(data) => {
                                        if let Err(e) = handle.write(&data).await {
                                            toast_api
                                                .error(
                                                    "Failed to export".to_string(),
                                                    ToastOptions::new()
                                                        .description(format!("{e}"))
                                                        .permanent(false)
                                                        .duration(ERROR_DURATION),
                                                );
                                        } else {
                                            toast_api
                                                .info(
                                                    "Export complete".to_string(),
                                                    ToastOptions::new().permanent(false).duration(INFO_DURATION),
                                                );
                                        }
                                    }
                                }
                            }
                        },
                        "Transaction Ledger"
                    }
                }
            }
        }

        h2 { font_size: "1rem", "Surplus Sale" }

        if datafile_open() {
            LoadedFile { loaded_file: datafile, configure_open }
        } else {
            "Nothing open..."
        }
    }
}

#[component]
pub fn NavbarIcon() -> Element {
    rsx! {
        svg {
            class: "navbar-expand-icon",
            view_box: "0 0 24 24",
            xmlns: "http://www.w3.org/2000/svg",
            polyline { points: "6 9 12 15 18 9" }
        }
    }
}
