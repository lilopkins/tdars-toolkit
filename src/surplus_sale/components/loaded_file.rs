use dioxus::prelude::*;
use dioxus_primitives::tabs::{TabContent, TabList, TabTrigger, Tabs};

use crate::surplus_sale::{
    components::{Auction, AuditLog, Configure, Reconciliation, SalesOverview},
    types::Datafile,
    NeedsSaving,
};

#[derive(PartialEq, Clone, Props)]
pub struct LoadedFileProps {
    configure_open: Signal<bool>,
    loaded_file: Datafile,
}

#[component]
pub fn LoadedFile(props: LoadedFileProps) -> Element {
    let mut datafile: Signal<Datafile> = use_context_provider(|| Signal::new(props.loaded_file));
    let mut needs_saving: Signal<NeedsSaving> = use_context();
    let configure_open = props.configure_open;

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
                // class: "tabs-content",
                index: 0usize,
                value: "overview".to_string(),

                AuditLog {}
            }
            TabContent {
                // class: "tabs-content",
                index: 1usize,
                value: "auction".to_string(),

                Auction {}
            }
            TabContent {
                // class: "tabs-content",
                index: 2usize,
                value: "reconcile".to_string(),

                Reconciliation {}
            }
            TabContent {
                // class: "tabs-content",
                index: 3usize,
                value: "sales".to_string(),

                SalesOverview {}
            }
        }

        Configure {
            open: configure_open,
            on_update: move |(cur, club_taking)| {
                datafile.write().set_currency(cur).set_club_taking(club_taking);
                needs_saving.set(NeedsSaving(true));
            },
            datafile,
        }
    }
}
