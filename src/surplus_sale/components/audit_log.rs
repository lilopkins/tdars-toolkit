use dioxus::prelude::*;
use dioxus_primitives::separator::Separator;

use crate::surplus_sale::types::Datafile;

#[component]
pub fn AuditLog() -> Element {
    let datafile: Signal<Datafile> = use_context();
    let audit_entries = use_memo(move || datafile.read().audit_log().clone());

    rsx! {
        div { display: "flex", flex_direction: "column", gap: ".6rem",
            for entry in audit_entries() {
                div {
                    key: "{entry}",
                    display: "flex",
                    flex_direction: "column",
                    gap: ".4rem",
                    span { class: "select", "{entry}" }
                    Separator {
                        class: "separator",
                        horizontal: true,
                        decorative: true,
                    }
                }
            }
        }
    }
}
