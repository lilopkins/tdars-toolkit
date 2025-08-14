use dioxus::prelude::*;
use dioxus_primitives::separator::Separator;

use crate::surplus_sale::{types::AuditEntry, DatafileHandle};

#[component]
pub fn AuditLog() -> Element {
    let datafile: Signal<DatafileHandle> = use_context();
    let audit_log: Signal<Vec<AuditEntry>> = use_signal(|| datafile().borrow().audit_log().clone());

    rsx! {
        for entry in &audit_log() {
            div { key: "{entry}",
                span { class: "select", "{entry}" }
                Separator { horizontal: true, decorative: true }
            }
        }
    }
}
