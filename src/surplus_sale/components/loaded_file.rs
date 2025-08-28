use dioxus::prelude::*;
use dioxus_primitives::tabs::{TabContent, TabList, TabTrigger, Tabs};

use crate::surplus_sale::{
    components::{
        configure::ConfigurationUpdateData, Auction, AuditLog, Configure, Reconciliation,
        SalesOverview,
    },
    types::Datafile,
    NeedsSaving,
};
#[cfg(feature = "escpos")]
use crate::types::ESCPOSDevice;

#[derive(PartialEq, Clone, Props)]
pub struct LoadedFileProps {
    configure_open: Signal<bool>,
    loaded_file: Signal<Datafile>,
}

#[component]
pub fn LoadedFile(props: LoadedFileProps) -> Element {
    let mut datafile: Signal<Datafile> = use_context_provider(|| props.loaded_file);
    #[cfg(feature = "escpos")]
    let mut escpos_device = use_context_provider(|| Signal::new(ESCPOSDevice(0x0000, 0x0000)));
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
            on_update: move |data: ConfigurationUpdateData| {
                datafile.write().set_currency(data.currency).set_club_taking(data.club_taking);
                #[cfg(feature = "escpos")]
                {
                    // deal with ESCPOD vendor and device
                    escpos_device.write().0 = data.escpos_vendor;
                    escpos_device.write().1 = data.escpos_device;
                }
                needs_saving.set(NeedsSaving(true));
            },
            datafile,
        }
    }
}
