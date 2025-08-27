use std::time::Duration;

use dioxus::{logger::tracing, prelude::*};
use dioxus_primitives::toast::{use_toast, ToastOptions};

use crate::hamfest_table::components::LoadedFile;
use crate::hamfest_table::types::Datafile;

#[component]
pub fn HamfestTable() -> Element {
    let toast_api = use_toast();
    let mut datafile: Signal<Option<Datafile>> = use_signal(|| None);
    let mut file_handle: Signal<Option<rfd::FileHandle>> = use_signal(|| None);
    let file_open = use_memo(move || datafile.read().is_some() && file_handle.read().is_some());
    use_effect(move || {
        // Save file on changes
        let handle = file_handle.peek();
        if let Some(datafile) = datafile.read().as_ref() {
            #[allow(
                clippy::unwrap_used,
                reason = "the format is guaranteed to be serializable"
            )]
            let data = serde_json::to_vec(&datafile).unwrap();
            if let Some(handle) = handle.clone() {
                spawn(async move {
                    if handle.write(&data).await.is_ok() {
                        toast_api.success(
                            "Automatically saved".to_string(),
                            ToastOptions::new()
                                .permanent(false)
                                .duration(Duration::from_secs(2)),
                        );
                    }
                });
            }
        }
    });

    rsx! {
        if file_open() {
            LoadedFile { datafile: datafile.map_mut(|v| v.as_ref().unwrap(), |v| v.as_mut().unwrap()) }
        } else {
            div { display: "flex", gap: ".5rem", flex_direction: "column",

                h1 { "Club Table" }

                button {
                    class: "fat wide button",
                    "data-style": "outline",
                    onclick: move |_| async move {
                        if let Some(handle) = rfd::AsyncFileDialog::new()
                            .add_filter("TDARS club table", &["tdars_club_table"])
                            .set_file_name("hamfest.tdars_club_table")
                            .save_file()
                            .await
                        {
                            tracing::info!("Creating new session...");
                            datafile.set(Some(Datafile::new()));
                            file_handle.set(Some(handle));
                        }
                    },
                    "New Session"
                }
                button {
                    class: "fat wide button",
                    "data-style": "outline",
                    onclick: move |_| async move {
                        if let Some(handle) = rfd::AsyncFileDialog::new()
                            .add_filter("TDARS club table", &["tdars_club_table"])
                            .pick_file()
                            .await
                        {
                            let data = handle.read().await;

                            match serde_json::from_slice::<Datafile>(&data) {
                                Ok(loaded_data) => {
                                    tracing::info!("Loaded session");
                                    datafile.set(Some(loaded_data));
                                    file_handle.set(Some(handle));
                                }
                                Err(e) => {
                                    toast_api
                                        .error(
                                            "Failed to load session".to_string(),
                                            ToastOptions::default().description(format!("{e}")),
                                        );
                                }
                            }
                        }
                    },
                    "Open Session"
                }
                button {
                    class: "fat wide button",
                    "data-style": "outline",
                    onclick: move |_| async move {
                        if let Some(handle) = rfd::AsyncFileDialog::new()
                            .add_filter("TDARS club table", &["tdars_club_table"])
                            .pick_file()
                            .await
                        {
                            let data = handle.read().await;

                            match serde_json::from_slice::<Datafile>(&data) {
                                Ok(loaded_data) => {
                                    tracing::info!("Loaded session");
                                    // TODO Create & Save log
                                }
                                Err(e) => {
                                    toast_api
                                        .error(
                                            "Failed to load session".to_string(),
                                            ToastOptions::default().description(format!("{e}")),
                                        );
                                }
                            }
                        }
                    },
                    "Transaction Log from Session"
                }
            }
        }
    }
}
