use std::cell::RefCell;

type DatafileHandle = RefCell<types::Datafile>;
struct NeedsSaving(bool);

mod components;
mod types;
mod views;

pub mod prelude {
    pub use super::views::SurplusSale;
}
