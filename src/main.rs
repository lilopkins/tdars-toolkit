#![warn(clippy::pedantic)]
#![deny(clippy::expect_used)]
#![deny(clippy::unwrap_used)]
#![cfg_attr(feature = "bundle", windows_subsystem = "windows")]

// The dioxus prelude contains a ton of common items used in dioxus apps. It's a good idea to import wherever you
// need dioxus
use dioxus::prelude::*;

use hamfest_table::prelude::*;
use surplus_sale::prelude::*;
use views::{Home, Navbar};

mod components;
mod types;
mod views;

/// Modules for the TDARS Toolkit
mod hamfest_table;
mod surplus_sale;

/// The Route enum is used to define the structure of internal routes in our app. All route enums need to derive
/// the [`Routable`] trait, which provides the necessary methods for the router to work.
///
/// Each variant represents a different URL pattern that can be matched by the router. If that pattern is matched,
/// the components for that route will be rendered.
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
        #[route("/")]
        Home {},
        #[route("/surplus-sale")]
        SurplusSale {},
        #[route("/hamfest-table")]
        HamfestTable {},
}

fn main() {
    // The `launch` function is the main entry point for a dioxus app. It takes a component and renders it with the platform feature
    // you have enabled
    dioxus::launch(App);
}

/// App is the main component of our app. Components are the building blocks of dioxus apps. Each component is a function
/// that takes some props and returns an Element. In this case, App takes no props because it is the root of our app.
///
/// Components should be annotated with `#[component]` to support props, better error messages, and autocomplete
#[component]
fn App() -> Element {
    // The `rsx!` macro lets us define HTML inside of rust. It expands to an Element with all of our HTML inside.
    rsx! {
        // The router component renders the route enum we defined above. It will handle synchronization of the URL and render
        // the layouts and components for the active route.
        Router::<Route> {}
    }
}
