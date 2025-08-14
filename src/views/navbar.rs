use crate::Route;
use dioxus::prelude::*;
use dioxus_primitives::toast::ToastProvider;

/// The Navbar component that will be rendered on all pages of our app since every page is under the layout.
///
///
/// This layout component wraps the UI of [Route::Home] and [Route::Blog] in a common navbar. The contents of the Home and Blog
/// routes will be rendered under the outlet inside this component
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
