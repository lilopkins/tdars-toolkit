//! The views module contains the components for all Layouts and Routes
//! for our app. Each layout and route in our [`Route`] enum will render
//! one of these components.

mod home;
pub use home::Home;

mod navbar;
pub use navbar::Navbar;
