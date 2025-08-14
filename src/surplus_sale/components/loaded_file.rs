use dioxus::prelude::*;
use dioxus_primitives::tabs::{TabContent, TabList, TabTrigger, Tabs};

use crate::surplus_sale::{components::AuditLog, DatafileHandle};

#[derive(PartialEq, Clone, Props)]
pub struct LoadedFileProps {
    loaded_file: DatafileHandle,
}

#[component]
pub fn LoadedFile(props: LoadedFileProps) -> Element {
    let _datafile: Signal<DatafileHandle> = use_context_provider(|| Signal::new(props.loaded_file));

    rsx! {
        Tabs {
            class: "tabs",
            default_value: "overview".to_string(),
            horizontal: true,

            TabList { class: "tabs-list",
                TabTrigger {
                    class: "tabs-trigger",
                    index: 0usize,
                    value: "overview".to_string(),
                    "Audit"
                }
                TabTrigger {
                    class: "tabs-trigger",
                    index: 1usize,
                    value: "auction".to_string(),
                    "Under the Hammer"
                }
                TabTrigger {
                    class: "tabs-trigger",
                    index: 2usize,
                    value: "reconcile".to_string(),
                    "Reconciliation"
                }
                TabTrigger {
                    class: "tabs-trigger",
                    index: 3usize,
                    value: "sales".to_string(),
                    "Sales Overview"
                }
            }

            TabContent {
                class: "tabs-content",
                index: 0usize,
                value: "overview".to_string(),

                AuditLog {}
            }
            TabContent {
                class: "tabs-content",
                index: 1usize,
                value: "auction".to_string(),
            }
            TabContent {
                class: "tabs-content",
                index: 2usize,
                value: "reconcile".to_string(),
            }
            TabContent {
                class: "tabs-content",
                index: 3usize,
                value: "sales".to_string(),
            }
        }
    }
}
