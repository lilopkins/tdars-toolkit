use dioxus::prelude::*;

use crate::Route;

#[component]
pub fn Home() -> Element {
    rsx! {
        div { display: "flex", flex_direction: "column", gap: "0.5rem",

            div { text_align: "center",
                h1 { margin_bottom: 0, "TDARS Toolkit" }
                p { margin_top: 0, "by Lily Hopkins 2E0HPS for the Telford & District ARS" }
            }

            Link { to: Route::SurplusSale {},
                button { class: "fat wide button", "data-style": "outline", "Run a Surplus Sale" }
            }
        }
    }
}
