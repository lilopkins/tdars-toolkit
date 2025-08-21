mod components;
mod export;
mod types;
mod views;

pub mod prelude {
    pub use super::views::SurplusSale;
}

#[derive(Copy, Clone)]
struct NeedsSaving(bool);

#[cfg(feature = "escpos")]
#[derive(Copy, Clone)]
struct ESCPOSDevice(u16, u16);
