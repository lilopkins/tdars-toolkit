use dioxus::prelude::*;

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
