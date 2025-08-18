mod components;
mod export;
mod types;
mod views;

pub mod prelude {
    pub use super::views::SurplusSale;
}

struct NeedsSaving(bool);
