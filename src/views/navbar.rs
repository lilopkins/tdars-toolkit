use crate::Route;
use dioxus::prelude::*;
use dioxus_primitives::toast::ToastProvider;

/// The Navbar component that will be rendered on all pages of our app
/// since every page is under the layout.
#[component]
pub fn Navbar() -> Element {
    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/styling/theme.css") }
        document::Link { rel: "stylesheet", href: asset!("/assets/styling/components.css") }
        document::Link { rel: "stylesheet", href: asset!("/assets/styling/main.css") }

        div { class: "wrapper no-select",
            ToastProvider { Outlet::<Route> {} }
        }
    }
}
